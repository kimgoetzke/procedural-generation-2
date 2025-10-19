use crate::constants::{CELL_LOCK_ERROR, CHUNK_SIZE};
use crate::coords::Point;
use crate::coords::point::{ChunkGrid, InternalGrid};
use crate::generation::lib::{LayeredPlane, TerrainType, TileType};
use crate::generation::object::lib::connection::get_connection_points;
use crate::generation::object::lib::{Cell, CellRef, Connection, TerrainState};
use crate::generation::resources::Climate;
use bevy::log::*;
use bevy::platform::collections::{HashMap, HashSet};
use bevy::reflect::Reflect;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

// TODO: Refactor ObjectGrid to not require two separate grids e.g. only use `CellRef` for pathfinding and wave function collapse
/// An [`ObjectGrid`] is a 2D grid of [`Cell`]s, each of which representing the possible states of objects that may be
/// spawned for the corresponding tile. The [`ObjectGrid`] is used to keep track of the state of each tile during the
/// object generation process and is discarded once the object generation process is complete as the outcome is
/// spawned as a child entity of the tile.
#[derive(Debug, Clone, Reflect)]
pub struct ObjectGrid {
  pub cg: Point<ChunkGrid>,
  #[reflect(ignore)]
  path_grid: Option<Vec<Vec<CellRef>>>,
  path: HashSet<Point<InternalGrid>>,
  object_grid: Vec<Vec<Cell>>,
  // TODO: Consider solving the below differently
  /// This [`Cell`] is used to represent out of bounds neighbours in the grid. It only allows [`ObjectName::Empty`] as
  /// permitted neighbours. Its purpose is to prevent "incomplete" multi-tile sprites.
  no_neighbours_tile: Cell,
  is_failure_log_level_increased: bool,
}

impl ObjectGrid {
  pub fn default(cg: Point<ChunkGrid>) -> Self {
    let object_grid: Vec<Vec<Cell>> = (0..CHUNK_SIZE)
      .map(|y| (0..CHUNK_SIZE).map(|x| Cell::new(x, y)).collect())
      .collect();

    ObjectGrid {
      cg,
      path_grid: None,
      path: HashSet::new(),
      object_grid,
      no_neighbours_tile: Cell::new(-1, -1),
      is_failure_log_level_increased: false,
    }
  }

  pub fn new_initialised(
    cg: Point<ChunkGrid>,
    climate: Climate,
    terrain_climate_state_map: &HashMap<(TerrainType, Climate), HashMap<TileType, Vec<TerrainState>>>,
    layered_plane: &LayeredPlane,
  ) -> Self {
    let mut grid = ObjectGrid::default(cg);
    let permitted_neighbours_of_empty = terrain_climate_state_map
      .get(&(TerrainType::Any, climate))
      .expect("Failed to find rule set for [Any] terrain type and [{:?}] climate combination")
      .get(&TileType::Fill)
      .expect("Failed to find rule set for [Fill] tile type")
      .clone();
    grid
      .no_neighbours_tile
      .override_possible_states(permitted_neighbours_of_empty);
    grid.initialise_cells(terrain_climate_state_map, climate, layered_plane);

    grid
  }

  /// Initialises object grid cells with terrain and tile type.
  fn initialise_cells(
    &mut self,
    terrain_climate_state_map: &HashMap<(TerrainType, Climate), HashMap<TileType, Vec<TerrainState>>>,
    climate: Climate,
    layered_plane: &LayeredPlane,
  ) {
    for tile in layered_plane.flat.data.iter().flatten().flatten() {
      let ig = tile.coords.internal_grid;
      let terrain = tile.terrain;
      let tile_type = tile.tile_type;
      // Uncomment is_monitored below for debugging purposes
      // Example: is_monitored = tile.coords.chunk_grid == Point::new_chunk_grid(15, 13) && ig == Point::new(15, 0);
      let is_monitored = false;
      if let Some(cell) = self.get_cell_mut(&ig) {
        let possible_states = terrain_climate_state_map
          .get(&(terrain, climate))
          .expect(
            format!(
              "Failed to find rule set for [{:?}] terrain type and [{:?}] climate combination",
              &terrain, &climate
            )
            .as_str(),
          )
          .get(&tile_type)
          .expect(format!("Failed to find rule set for [{:?}] tile type", &tile_type).as_str())
          .clone();
        let lower_tile_data = layered_plane
          .planes
          .iter()
          .filter_map(|plane| {
            let terrain_as_usize = tile.terrain as usize;
            if is_monitored {
              debug!(
                "At {:?}, plane with layer [{}] is included: [{}] because [{}] is [{:?}]",
                tile.coords,
                plane.layer.unwrap_or(usize::MAX),
                plane.layer.unwrap_or(usize::MIN) < terrain_as_usize,
                tile.terrain,
                terrain_as_usize,
              );
            }
            if let Some(plane_layer_as_usize) = plane.layer {
              if plane_layer_as_usize < terrain_as_usize {
                return Some(plane);
              }
            }
            return None;
          })
          .filter_map(|plane| plane.get_tile(ig).and_then(|t| Some((t.terrain, t.tile_type))))
          .collect::<Vec<(TerrainType, TileType)>>();
        cell.initialise(terrain, tile_type, &possible_states, lower_tile_data.clone(), is_monitored);
        if is_monitored {
          debug!(
            "Initialised {:?} as a [{:?}] [{:?}] cell with {:?} state(s)",
            tile.coords,
            tile.terrain,
            tile.tile_type,
            cell.get_possible_states().len(),
          );
          cell.log_tiles_below();
          debug!(
            "- Lower tile data for {:?} was: {:?}",
            ig,
            lower_tile_data
              .iter()
              .map(|(t, tt)| format!("{:?} {:?}", t, tt))
              .collect::<Vec<String>>()
          );
        }
      } else {
        error!("Failed to find cell to initialise at {:?}", ig);
      }
    }
  }

  /// Initialises the path finding grid by populating it with strong references to the respective [`Cell`]s, if
  /// it has not been initialised yet. Then, populates the neighbours for each cell.
  pub fn initialise_path_grid(&mut self) {
    if self.path_grid.is_none() {
      self.path_grid = Some(
        (0..CHUNK_SIZE)
          .map(|y| {
            (0..CHUNK_SIZE)
              .map(|x| {
                if let Some(existing_cell) = self.object_grid.get(y as usize).and_then(|row| row.get(x as usize)) {
                  return Arc::new(Mutex::new(existing_cell.clone()));
                }

                Arc::new(Mutex::new(Cell::new(x, y)))
              })
              .collect()
          })
          .collect(),
      );
    }
    if let Some(grid) = &mut self.path_grid {
      for y in 0..grid.len() {
        for x in 0..grid[y].len() {
          let cell_ref = &grid[y][x];
          let ig = cell_ref.lock().expect(CELL_LOCK_ERROR).ig;
          let mut neighbours: Vec<CellRef> = Vec::new();

          for (dx, dy) in [(0, 1), (-1, 0), (1, 0), (0, -1)] {
            let nx = ig.x + dx;
            let ny = ig.y + dy;
            if nx >= 0 && ny >= 0 {
              if let Some(row) = grid.get(ny as usize) {
                if let Some(neighbour_ref) = row.get(nx as usize) {
                  neighbours.push(neighbour_ref.clone());
                }
              }
            }
          }

          let mut cell_guard = cell_ref.try_lock().expect(CELL_LOCK_ERROR);
          cell_guard.add_neighbours(neighbours);
          cell_guard.calculate_is_walkable();
        }
      }
    }
  }

  pub fn get_cell_ref(&self, point: &Point<InternalGrid>) -> Option<&CellRef> {
    if self.path_grid.is_none() {
      error!("You're trying to get a cell reference from an uninitialised path grid - this is a bug!");
      return None;
    }
    self
      .path_grid
      .as_ref()?
      .iter()
      .flatten()
      .find(|cell| cell.lock().expect(CELL_LOCK_ERROR).ig == *point)
  }

  // TODO: Use weak references in Cell to make future memory leak less likely
  /// Resets the path grid by clearing all references in each cell. This is required but not sufficient for the grid to
  /// be reused for a new pathfinding operation. The path finding grid will have to be re-initialised again.
  /// As long as [`Cell`] uses strong references to its neighbours of for any connections (both of which it should not)
  /// this method must also be called prior finishing the pathfinding operation to prevent memory leaks.
  pub fn reset_path_grid(&mut self) {
    if let Some(grid) = &mut self.path_grid {
      for row in grid {
        for cell_ref in row {
          if let Ok(mut cell) = cell_ref.try_lock() {
            cell.clear_references();
          }
        }
      }
    }
  }

  /// Sets the final path found by the pathfinding algorithm. This can be used by subsequent generation steps.
  pub fn set_generated_path(&mut self, path: HashSet<Point<InternalGrid>>) {
    self.path = path;
  }

  /// Returns a reference to the final path found by the pathfinding algorithm.
  pub fn get_generated_path(&self) -> &HashSet<Point<InternalGrid>> {
    &self.path
  }

  /// Validates the object grid to ensure it is in a consistent state. Assumed to be called prior to starting the wave
  /// function collapse algorithm.
  /// # Panics
  /// If a cell must be updated but the update fails, this method will panic.
  pub fn validate(&mut self) {
    let mut collapsed_cells: VecDeque<Cell> = VecDeque::new();
    let mut edge_cells: VecDeque<Cell> = VecDeque::new();
    self.object_grid.iter().flatten().for_each(|c| {
      if c.is_collapsed() {
        collapsed_cells.push_back(c.clone());
      } else if c.ig.is_touching_edge() {
        edge_cells.push_back(c.clone());
      }
    });
    let cg = self.cg;
    let mut i = 0;
    self.update_neighbours_of_collapsed_cells(&mut collapsed_cells, &cg, &mut i);
    self.update_edge_cells(&mut edge_cells, &cg, &mut i);
    debug!("Validated object grid {} and made [{}] updates to cells' states", cg, i);
  }

  /// Updates all neighbours of already collapsed cells (and recurses over all neighbours of any updated the neighbour)
  /// to ensure their states are valid. This is important because we're allowing some cells to be pre-collapsed (e.g.
  /// for paths) and we need to ensure the rest of the grid is valid before starting the wave function collapse
  /// algorithm.
  fn update_neighbours_of_collapsed_cells(
    &mut self,
    collapsed_cells: &mut VecDeque<Cell>,
    cg: &Point<ChunkGrid>,
    i: &mut i32,
  ) {
    while let Some(cell) = collapsed_cells.pop_front() {
      for (connection, neighbour_ig) in get_connection_points(&cell.ig) {
        if let Some(neighbour) = self.get_cell(&neighbour_ig) {
          if neighbour.is_collapsed() {
            continue;
          }
          match neighbour.clone_and_reduce(&cell, &connection, false) {
            Ok((true, updated_neighbour)) => {
              trace!(
                "Validating object grid {}: Reduced possible states of {:?} from {:?} to {:?}",
                cg,
                neighbour_ig,
                neighbour.get_possible_states().len(),
                updated_neighbour.get_possible_states().len(),
              );
              self.set_cell(updated_neighbour.clone());
              collapsed_cells.push_back(updated_neighbour);
              *i += 1;
            }
            Ok((false, _)) => {}
            Err(_) => {
              panic!(
                "Validating object grid {}: Failed to reduce neighbour at {:?} of collapsed cell at {:?}",
                cg, neighbour_ig, cell.ig,
              );
            }
          }
        }
      }
    }
  }

  /// Updates all cells that touch any edge of the grid to ensure their states are valid.
  fn update_edge_cells(&mut self, edge_cells: &mut VecDeque<Cell>, cg: &Point<ChunkGrid>, i: &mut i32) {
    while let Some(cell) = edge_cells.pop_front() {
      let edge_connections: Vec<Connection> = get_connection_points(&cell.ig)
        .iter()
        .filter_map(|(c, p)| if p.is_outside_grid() { Some(c) } else { None })
        .cloned()
        .collect();
      for connection in edge_connections {
        match cell.clone_and_reduce(&self.no_neighbours_tile, &connection, false) {
          Ok((true, updated_cell)) => {
            trace!(
              "Validating object grid {}: Reduced possible states of {:?} from {:?} to {:?}",
              cg,
              cell.ig,
              cell.get_possible_states().len(),
              updated_cell.get_possible_states().len(),
            );
            self.set_cell(updated_cell.clone());
            *i += 1;
          }
          Ok((false, _)) => {}
          Err(_) => {
            panic!(
              "Validating object grid {}: Failed to reduce edge cell at {:?} of collapsed cell at {:?}",
              cg, cell, cell.ig,
            );
          }
        }
      }
    }
  }

  pub fn get_neighbours(&mut self, cell: &Cell) -> Vec<(Connection, &Cell)> {
    let point = cell.ig;
    let points: Vec<(Connection, Point<InternalGrid>)> = get_connection_points(&point).into_iter().collect();
    let mut neighbours = vec![];
    for (direction, point) in points {
      if let Some(cell) = self.object_grid.iter().flatten().filter(|cell| cell.ig == point).next() {
        neighbours.push((direction, cell));
      } else {
        neighbours.push((direction, &self.no_neighbours_tile));
      }
    }

    neighbours
  }

  pub fn get_cell(&self, ig: &Point<InternalGrid>) -> Option<&Cell> {
    self.object_grid.iter().flatten().find(|cell| cell.ig == *ig)
  }

  pub fn get_cell_mut(&mut self, ig: &Point<InternalGrid>) -> Option<&mut Cell> {
    self.object_grid.iter_mut().flatten().find(|cell| cell.ig == *ig)
  }

  /// Replaces the [`Cell`] at the given point with the provided [`Cell`].
  pub fn set_cell(&mut self, cell: Cell) {
    if let Some(existing_cell) = self.object_grid.iter_mut().flatten().find(|c| c.ig == cell.ig) {
      *existing_cell = cell;
    } else {
      error!("Failed to find cell to update at {:?}", cell.ig);
    }
  }

  pub fn calculate_total_entropy(&self) -> i32 {
    self.object_grid.iter().flatten().map(|cell| cell.get_entropy() as i32).sum()
  }

  pub fn get_cells_with_lowest_entropy(&self) -> Vec<&Cell> {
    let mut lowest_entropy = usize::MAX;
    let mut lowest_entropy_cells = vec![];
    for cell in self.object_grid.iter().flatten() {
      if !cell.is_collapsed() {
        let entropy = cell.get_entropy();
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
    self.object_grid = other.object_grid.clone();
  }

  pub fn is_failure_log_level_increased(&self) -> bool {
    self.is_failure_log_level_increased
  }

  pub fn increase_failure_log_level(&mut self) {
    self.is_failure_log_level_increased = true;
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  impl ObjectGrid {
    pub fn default_walkable(cg: Point<ChunkGrid>) -> Self {
      let mut grid = ObjectGrid::default(cg);
      for row in &mut grid.object_grid {
        for cell in row {
          cell.initialise(TerrainType::Land2, TileType::Fill, &vec![], vec![], false);
        }
      }

      grid
    }
  }
}
