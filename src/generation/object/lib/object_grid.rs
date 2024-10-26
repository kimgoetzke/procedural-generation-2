use crate::constants::CHUNK_SIZE;
use crate::coords::point::ChunkGrid;
use crate::coords::Point;
use crate::generation::lib::{TerrainType, TileData, TileType};
use crate::generation::object::lib::connection_type::get_connection_points;
use crate::generation::object::lib::{Cell, Connection, ObjectName};
use crate::generation::resources::TerrainState;
use bevy::log::*;
use bevy::reflect::Reflect;
use bevy::utils::HashMap;

#[derive(Debug, Clone, Reflect)]
pub struct ObjectGrid {
  #[reflect(ignore)]
  pub grid: Vec<Vec<Cell>>,
}

// TODO: Add constraints from neighbouring chunk tiles somehow
impl ObjectGrid {
  fn new_uninitialised() -> Self {
    let grid: Vec<Vec<Cell>> = (0..CHUNK_SIZE)
      .map(|y| (0..CHUNK_SIZE).map(|x| Cell::new(x, y)).collect())
      .collect();
    ObjectGrid { grid }
  }

  pub fn new_initialised(
    terrain_rules: &HashMap<TerrainType, Vec<TerrainState>>,
    tile_type_rules: &HashMap<TileType, Vec<ObjectName>>,
    tile_data: &Vec<TileData>,
  ) -> Self {
    let mut grid = ObjectGrid::new_uninitialised();
    for data in tile_data.iter() {
      let cg = data.flat_tile.coords.chunk_grid;
      let terrain = data.flat_tile.terrain;
      let tile_type = data.flat_tile.tile_type;
      if let Some(cell) = grid.get_cell_mut(&cg) {
        let relevant_rules = resolve_rules(tile_type, terrain_rules, tile_type_rules, terrain);
        cell.initialise(terrain, &relevant_rules);
        trace!(
          "Initialised cg{:?} as a [{:?}] [{:?}] cell with {:?} state(s)",
          cg,
          data.flat_tile.terrain,
          data.flat_tile.tile_type,
          cell.possible_states.len()
        );
      } else {
        error!("Failed to find cell to initialise at cg{:?}", cg);
      }
    }

    grid
  }

  pub fn get_neighbours(&mut self, point: &Point<ChunkGrid>) -> Vec<(Connection, &Cell)> {
    let points: Vec<_> = get_connection_points(point).into_iter().collect();
    let mut neighbours = vec![];
    for (direction, point) in points {
      if let Some(cell) = self.grid.iter().flatten().filter(|cell| cell.cg == point).next() {
        neighbours.push((direction, cell));
      }
    }
    trace!("Found {} neighbours for cg{:?}", neighbours.len(), point);

    neighbours
  }

  pub fn get_cell(&self, point: &Point<ChunkGrid>) -> Option<&Cell> {
    self.grid.iter().flatten().find(|cell| cell.cg == *point)
  }

  pub fn get_cell_mut(&mut self, point: &Point<ChunkGrid>) -> Option<&mut Cell> {
    self.grid.iter_mut().flatten().find(|cell| cell.cg == *point)
  }

  /// Replaces the `Cell` at the given point with the provided `Cell`.
  pub fn set_cell(&mut self, cell: Cell) {
    if let Some(existing_cell) = self.grid.iter_mut().flatten().find(|c| c.cg == cell.cg) {
      *existing_cell = cell;
    } else {
      error!("Failed to find cell to update at cg{:?}", cell.cg);
    }
  }

  pub fn calculate_total_entropy(&self) -> i32 {
    self.grid.iter().flatten().map(|cell| cell.entropy as i32).sum()
  }

  pub fn get_cells_with_lowest_entropy(&self) -> Vec<&Cell> {
    let mut lowest_entropy = usize::MAX;
    let mut lowest_entropy_cells = vec![];
    for cell in self.grid.iter().flatten() {
      if !cell.is_collapsed {
        let entropy = cell.entropy;
        if entropy < lowest_entropy {
          lowest_entropy = entropy;
          lowest_entropy_cells = vec![cell];
        } else if entropy == lowest_entropy {
          lowest_entropy_cells.push(cell);
        }
      }
    }
    trace!(
      "Found {} cell(s) with lowest entropy of {}",
      lowest_entropy_cells.len(),
      lowest_entropy
    );

    lowest_entropy_cells
  }

  pub fn restore_from_snapshot(&mut self, other: &ObjectGrid) {
    self.grid = other.grid.clone();
  }
}

// TODO: Make resolving rules for each tile type part of the app initialisation process
//  instead of repeating for each tile during the object generation process
fn resolve_rules(
  tile_type: TileType,
  terrain_rules: &HashMap<TerrainType, Vec<TerrainState>>,
  tile_type_rules: &HashMap<TileType, Vec<ObjectName>>,
  terrain: TerrainType,
) -> Vec<TerrainState> {
  let relevant_terrain_rules = terrain_rules
    .get(&terrain)
    .expect(format!("Failed to find rule set for [{:?}] terrain type", &terrain).as_str());
  let relevant_tile_type_rules = tile_type_rules
    .get(&tile_type)
    .expect(format!("Failed to find rule set for [{:?}] tile type", &tile_type).as_str());

  let mut resolved_rules = vec![];
  for terrain_rule in relevant_terrain_rules {
    if relevant_tile_type_rules.contains(&terrain_rule.name) {
      resolved_rules.push(terrain_rule.clone());
    }
  }

  debug!(
    "Resolved {} rules for this [{:?}] tile from {:?} [{}] terrain rules and {:?} tile type rules: {:?}",
    resolved_rules.len(),
    tile_type,
    terrain_rules.len(),
    terrain,
    tile_type_rules.len(),
    resolved_rules.iter().map(|r| r.name).collect::<Vec<ObjectName>>()
  );

  resolved_rules
}
