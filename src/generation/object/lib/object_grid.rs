use crate::constants::CHUNK_SIZE;
use crate::coords::Point;
use crate::coords::point::{ChunkGrid, InternalGrid};
use crate::generation::lib::{TerrainType, TileType};
use crate::generation::object::lib::connection_type::get_connection_points;
use crate::generation::object::lib::tile_data::TileData;
use crate::generation::object::lib::{Cell, Connection, ObjectName};
use crate::generation::resources::TerrainState;
use bevy::log::*;
use bevy::platform::collections::HashMap;
use bevy::reflect::Reflect;

/// An `ObjectGrid` is a 2D grid of `Cell`s, each of which representing the possible states of objects that may be
/// spawned for the corresponding tile. The `ObjectGrid` is used to keep track of the state of each tile during the
/// object generation process and is discarded once the object generation process is complete as the outcome is
/// spawned as a child entity of the tile.
#[derive(Debug, Clone, Reflect)]
pub struct ObjectGrid {
  pub cg: Point<ChunkGrid>,
  #[reflect(ignore)]
  pub grid: Vec<Vec<Cell>>,
  // TODO: Consider solving the below differently
  /// This `Cell` is used to represent out of bounds neighbours in the grid. It only allows `ObjectName::Empty` as
  /// permitted neighbours. Its purpose is to prevent "incomplete" multi-tile sprites.
  pub no_neighbours_tile: Cell,
}

impl ObjectGrid {
  fn new_uninitialised(cg: Point<ChunkGrid>) -> Self {
    let grid: Vec<Vec<Cell>> = (0..CHUNK_SIZE)
      .map(|y| (0..CHUNK_SIZE).map(|x| Cell::new(x, y)).collect())
      .collect();
    let mut no_neighbours_tile = Cell::new(-1, -1);
    no_neighbours_tile.possible_states = vec![TerrainState {
      name: ObjectName::Empty,
      index: 0,
      weight: 1,
      permitted_neighbours: vec![
        (Connection::Top, vec![ObjectName::Empty]),
        (Connection::Right, vec![ObjectName::Empty]),
        (Connection::Bottom, vec![ObjectName::Empty]),
        (Connection::Left, vec![ObjectName::Empty]),
      ],
    }];
    ObjectGrid {
      cg,
      grid,
      no_neighbours_tile,
    }
  }

  pub fn new_initialised(
    cg: Point<ChunkGrid>,
    terrain_rules: &HashMap<TerrainType, Vec<TerrainState>>,
    tile_type_rules: &HashMap<TileType, Vec<ObjectName>>,
    tile_data: &Vec<TileData>,
  ) -> Self {
    let mut grid = ObjectGrid::new_uninitialised(cg);
    for data in tile_data.iter() {
      let ig = data.flat_tile.coords.internal_grid;
      let terrain = data.flat_tile.terrain;
      let tile_type = data.flat_tile.tile_type;
      if let Some(cell) = grid.get_cell_mut(&ig) {
        let relevant_rules = resolve_rules(tile_type, terrain_rules, tile_type_rules, terrain);
        cell.initialise(terrain, tile_type, &relevant_rules);
        trace!(
          "Initialised {:?} as a [{:?}] [{:?}] cell with {:?} state(s)",
          ig,
          data.flat_tile.terrain,
          data.flat_tile.tile_type,
          cell.possible_states.len()
        );
      } else {
        error!("Failed to find cell to initialise at {:?}", ig);
      }
    }

    grid
  }

  pub fn get_neighbours(&mut self, cell: &Cell) -> Vec<(Connection, &Cell)> {
    let point = cell.ig;
    let points: Vec<_> = get_connection_points(&point).into_iter().collect();
    let mut neighbours = vec![];
    for (direction, point) in points {
      if let Some(cell) = self.grid.iter().flatten().filter(|cell| cell.ig == point).next() {
        neighbours.push((direction, cell));
      } else {
        neighbours.push((direction, &self.no_neighbours_tile));
      }
    }
    trace!("Found {} neighbours for {:?}", neighbours.len(), point);

    neighbours
  }

  pub fn get_cell(&self, point: &Point<InternalGrid>) -> Option<&Cell> {
    self.grid.iter().flatten().find(|cell| cell.ig == *point)
  }

  pub fn get_cell_mut(&mut self, point: &Point<InternalGrid>) -> Option<&mut Cell> {
    self.grid.iter_mut().flatten().find(|cell| cell.ig == *point)
  }

  /// Replaces the `Cell` at the given point with the provided `Cell`.
  pub fn set_cell(&mut self, cell: Cell) {
    if let Some(existing_cell) = self.grid.iter_mut().flatten().find(|c| c.ig == cell.ig) {
      *existing_cell = cell;
    } else {
      error!("Failed to find cell to update at {:?}", cell.ig);
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

  trace!(
    "Resolved {} rule(s) for this [{:?}] tile from {:?} [{}] terrain rules and {:?} tile type rules: {:?}",
    resolved_rules.len(),
    tile_type,
    terrain_rules.len(),
    terrain,
    tile_type_rules.len(),
    resolved_rules.iter().map(|r| r.name).collect::<Vec<ObjectName>>()
  );

  resolved_rules
}
