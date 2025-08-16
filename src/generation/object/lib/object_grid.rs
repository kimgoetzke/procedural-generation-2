use crate::constants::CHUNK_SIZE;
use crate::coords::Point;
use crate::coords::point::{ChunkGrid, InternalGrid};
use crate::generation::lib::{LayeredPlane, TerrainType, TileType};
use crate::generation::object::lib::connection::get_connection_points;
use crate::generation::object::lib::{Cell, CellRef, Connection, ObjectName, TerrainState};
use bevy::log::*;
use bevy::platform::collections::HashMap;
use bevy::reflect::Reflect;
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
  pub path_grid: Option<Vec<Vec<CellRef>>>,
  pub object_grid: Vec<Vec<Cell>>,
  // TODO: Consider solving the below differently
  /// This [`Cell`] is used to represent out of bounds neighbours in the grid. It only allows [`ObjectName::Empty`] as
  /// permitted neighbours. Its purpose is to prevent "incomplete" multi-tile sprites.
  no_neighbours_tile: Cell,
}

impl ObjectGrid {
  fn new_uninitialised(cg: Point<ChunkGrid>) -> Self {
    let object_grid: Vec<Vec<Cell>> = (0..CHUNK_SIZE)
      .map(|y| (0..CHUNK_SIZE).map(|x| Cell::new(x, y)).collect())
      .collect();
    let mut no_neighbours_tile = Cell::new(-1, -1);
    no_neighbours_tile.override_possible_states(vec![TerrainState::new_with_no_neighbours(ObjectName::Empty, 0, 1)]);

    ObjectGrid {
      cg,
      path_grid: None,
      object_grid,
      no_neighbours_tile,
    }
  }

  pub fn new_initialised(
    cg: Point<ChunkGrid>,
    terrain_state_map: &HashMap<TerrainType, HashMap<TileType, Vec<TerrainState>>>,
    layered_plane: &LayeredPlane,
  ) -> Self {
    let mut grid = ObjectGrid::new_uninitialised(cg);
    grid.initialise_cells(terrain_state_map, layered_plane);

    grid
  }

  /// Initialises object grid cells with terrain and tile type.
  fn initialise_cells(
    &mut self,
    terrain_state_map: &HashMap<TerrainType, HashMap<TileType, Vec<TerrainState>>>,
    layered_plane: &LayeredPlane,
  ) {
    for tile in layered_plane.flat.data.iter().flatten().flatten() {
      let ig = tile.coords.internal_grid;
      let terrain = tile.terrain;
      let tile_type = tile.tile_type;
      let is_monitored = tile.coords.chunk_grid == Point::new_chunk_grid(15, 17) && ig == Point::new(10, 0);
      if let Some(cell) = self.get_cell_mut(&ig) {
        let possible_states = terrain_state_map
          .get(&terrain)
          .expect(format!("Failed to find rule set for [{:?}] terrain type", &terrain).as_str())
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
        cell.initialise(terrain, tile_type, &possible_states, lower_tile_data.clone());
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
  pub(crate) fn initialise_path_grid(&mut self) {
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
          let ig = cell_ref.lock().expect("Failed to lock cell").ig;
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

          let mut cell_guard = cell_ref.try_lock().expect("Failed to lock cell");
          cell_guard.add_neighbours(neighbours);
          cell_guard.calculate_is_walkable();
        }
      }
    }
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

  pub fn get_neighbours(&mut self, cell: &Cell) -> Vec<(Connection, &Cell)> {
    let point = cell.ig;
    let points: Vec<_> = get_connection_points(&point).into_iter().collect();
    let mut neighbours = vec![];
    for (direction, point) in points {
      if let Some(cell) = self.object_grid.iter().flatten().filter(|cell| cell.ig == point).next() {
        neighbours.push((direction, cell));
      } else {
        neighbours.push((direction, &self.no_neighbours_tile));
      }
    }
    trace!("Found {} neighbours for {:?}", neighbours.len(), point);

    neighbours
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
      .find(|cell| cell.lock().expect("Failed to lock cell").ig == *point)
  }

  pub fn get_cell(&self, point: &Point<InternalGrid>) -> Option<&Cell> {
    self.object_grid.iter().flatten().find(|cell| cell.ig == *point)
  }

  pub fn get_cell_mut(&mut self, point: &Point<InternalGrid>) -> Option<&mut Cell> {
    self.object_grid.iter_mut().flatten().find(|cell| cell.ig == *point)
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
}
