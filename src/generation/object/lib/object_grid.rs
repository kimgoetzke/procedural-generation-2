use crate::constants::CHUNK_SIZE;
use crate::coords::Point;
use crate::coords::point::{ChunkGrid, InternalGrid};
use crate::generation::lib::{TerrainType, TileType};
use crate::generation::object::lib::connection::get_connection_points;
use crate::generation::object::lib::{Cell, CellRef, Connection, ObjectName, TerrainState, TileData};
use bevy::log::*;
use bevy::platform::collections::HashMap;
use bevy::reflect::Reflect;
use std::sync::{Arc, Mutex};

/// An [`ObjectGrid`] is a 2D grid of [`Cell`]s, each of which representing the possible states of objects that may be
/// spawned for the corresponding tile. The [`ObjectGrid`] is used to keep track of the state of each tile during the
/// object generation process and is discarded once the object generation process is complete as the outcome is
/// spawned as a child entity of the tile.
#[derive(Debug, Clone, Reflect)]
pub struct ObjectGrid {
  pub cg: Point<ChunkGrid>,
  #[reflect(ignore)]
  pub path_grid: Vec<Vec<CellRef>>,
  // TODO: Remove the field below and only use `CellRef` for pathfinding and wave function collapse
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
    let path_grid: Vec<Vec<CellRef>> = (0..CHUNK_SIZE)
      .map(|y| (0..CHUNK_SIZE).map(|x| Arc::new(Mutex::new(Cell::new(x, y)))).collect())
      .collect();
    let mut no_neighbours_tile = Cell::new(-1, -1);
    no_neighbours_tile.override_possible_states(vec![TerrainState {
      name: ObjectName::Empty,
      index: 0,
      weight: 1,
      permitted_neighbours: vec![
        (Connection::Top, vec![ObjectName::Empty]),
        (Connection::Right, vec![ObjectName::Empty]),
        (Connection::Bottom, vec![ObjectName::Empty]),
        (Connection::Left, vec![ObjectName::Empty]),
      ],
    }]);
    ObjectGrid {
      cg,
      path_grid,
      object_grid,
      no_neighbours_tile,
    }
  }

  pub fn new_initialised(
    cg: Point<ChunkGrid>,
    terrain_state_map: &HashMap<TerrainType, HashMap<TileType, Vec<TerrainState>>>,
    tile_data: &Vec<TileData>,
  ) -> Self {
    let mut grid = ObjectGrid::new_uninitialised(cg);

    // Initialise path finding grid by populate neighbours for each cell
    for y in 0..grid.path_grid.len() {
      for x in 0..grid.path_grid[y].len() {
        let cell_ref = &grid.path_grid[y][x];
        let ig = cell_ref.lock().expect("Failed to lock CellRef").ig;
        let mut neighbours: Vec<CellRef> = Vec::new();

        for (dx, dy) in [(-1, 1), (0, 1), (1, 1), (-1, 0), (1, 0), (-1, -1), (0, -1), (1, -1)] {
          let nx = ig.x + dx;
          let ny = ig.y + dy;
          if nx >= 0 && ny >= 0 {
            if let Some(row) = grid.path_grid.get(ny as usize) {
              if let Some(neighbour_ref) = row.get(nx as usize) {
                neighbours.push(neighbour_ref.clone());
              }
            }
          }
        }

        let mut cell = cell_ref.lock().expect("Failed to lock cell");
        cell.add_neighbours(neighbours);
      }
    }

    // Initialise object grid cells with terrain and tile type
    for data in tile_data.iter() {
      let ig = data.flat_tile.coords.internal_grid;
      let terrain = data.flat_tile.terrain;
      let tile_type = data.flat_tile.tile_type;
      if let Some(cell) = grid.get_cell_mut(&ig) {
        let possible_states = terrain_state_map
          .get(&terrain)
          .expect(format!("Failed to find rule set for [{:?}] terrain type", &terrain).as_str())
          .get(&tile_type)
          .expect(format!("Failed to find rule set for [{:?}] tile type", &tile_type).as_str())
          .clone();
        cell.initialise(terrain, tile_type, &possible_states);
        trace!(
          "Initialised {:?} as a [{:?}] [{:?}] cell with {:?} state(s)",
          ig,
          data.flat_tile.terrain,
          data.flat_tile.tile_type,
          cell.get_possible_states().len()
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
    self
      .path_grid
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
