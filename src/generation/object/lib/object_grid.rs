use crate::constants::{CELL_LOCK_ERROR, CHUNK_SIZE};
use crate::coords::Point;
use crate::coords::point::{ChunkGrid, InternalGrid};
use crate::generation::lib::{LayeredPlane, TerrainType, TileType};
use crate::generation::object::lib::cell::PropagationFailure;
use crate::generation::object::lib::connection::get_connection_points;
use crate::generation::object::lib::{Cell, CellRef, Connection, ObjectGridSnapshot, TerrainState};
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

    Self {
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
    let mut grid = Self::default(cg);
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
          .unwrap_or_else(|| {
            panic!(
              "Failed to find rule set for [{:?}] terrain type and [{:?}] climate combination",
              &terrain, &climate
            )
          })
          .get(&tile_type)
          .unwrap_or_else(|| panic!("Failed to find rule set for [{:?}] tile type", &tile_type))
          .clone();
        let lower_tile_data = layered_plane
          .planes
          .iter()
          .inspect(|plane| {
            if is_monitored {
              let terrain_as_usize = tile.terrain as usize;
              debug!(
                "At {:?}, plane with layer [{}] is included: [{}] because [{}] is [{:?}]",
                tile.coords,
                plane.layer.unwrap_or(usize::MAX),
                plane.layer.unwrap_or(usize::MIN) < terrain_as_usize,
                tile.terrain,
                terrain_as_usize,
              );
            }
          })
          .filter(|plane| {
            let terrain_as_usize = tile.terrain as usize;
            if let Some(plane_layer_as_usize) = plane.layer
              && plane_layer_as_usize < terrain_as_usize
            {
              true
            } else {
              false
            }
          })
          .filter_map(|plane| plane.get_tile(ig).map(|t| (t.terrain, t.tile_type)))
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
            if nx >= 0
              && ny >= 0
              && let Some(row) = grid.get(ny as usize)
              && let Some(neighbour_ref) = row.get(nx as usize)
            {
              neighbours.push(neighbour_ref.clone());
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
  pub const fn get_generated_path(&self) -> &HashSet<Point<InternalGrid>> {
    &self.path
  }

  /// Validates the object grid to ensure it is in a consistent state. Assumed to be called prior to starting the wave
  /// function collapse algorithm.
  /// # Panics
  /// If a cell must be updated but the update fails, this method will panic.
  pub fn validate(&mut self) {
    let mut collapsed_cell_igs: VecDeque<Point<InternalGrid>> = VecDeque::new();
    let mut edge_cell_igs: VecDeque<Point<InternalGrid>> = VecDeque::new();
    self.object_grid.iter().flatten().for_each(|cell| {
      if cell.is_collapsed() {
        collapsed_cell_igs.push_back(cell.ig);
      } else if cell.ig.is_touching_edge() {
        edge_cell_igs.push_back(cell.ig);
      }
    });
    let cg = self.cg;
    let mut i = 0;
    self.update_neighbours_of_collapsed_cells(&mut collapsed_cell_igs, &cg, &mut i);
    self.update_edge_cells(&mut edge_cell_igs, &cg, &mut i);
    debug!("Validated object grid {} and made [{}] updates to cells' states", cg, i);
  }

  /// Updates all neighbours of already collapsed cells (and recurses over all neighbours of any updated the neighbour)
  /// to ensure their states are valid. This is important because we're allowing some cells to be pre-collapsed (e.g.
  /// for paths) and we need to ensure the rest of the grid is valid before starting the wave function collapse
  /// algorithm.
  fn update_neighbours_of_collapsed_cells(
    &mut self,
    collapsed_cell_igs: &mut VecDeque<Point<InternalGrid>>,
    cg: &Point<ChunkGrid>,
    i: &mut i32,
  ) {
    while let Some(reference_ig) = collapsed_cell_igs.pop_front() {
      for (connection, neighbour_ig) in get_connection_points(&reference_ig) {
        if neighbour_ig.is_outside_grid() || self.get_cell(&neighbour_ig).is_none_or(Cell::is_collapsed) {
          continue;
        }
        let previous_entropy = self.get_cell(&neighbour_ig).map(Cell::get_entropy).unwrap_or_default();
        match self.reduce_neighbour(&reference_ig, &neighbour_ig, &connection, false) {
          Ok(true) => {
            let updated_entropy = self.get_cell(&neighbour_ig).map(Cell::get_entropy).unwrap_or_default();
            trace!(
              "Validating object grid {}: Reduced possible states of {:?} from {:?} to {:?}",
              cg, neighbour_ig, previous_entropy, updated_entropy,
            );
            collapsed_cell_igs.push_back(neighbour_ig);
            *i += 1;
          }
          Ok(false) => {}
          Err(_) => {
            panic!(
              "Validating object grid {}: Failed to reduce neighbour at {:?} of collapsed cell at {:?}",
              cg, neighbour_ig, reference_ig,
            );
          }
        }
      }
    }
  }

  /// Updates all cells that touch any edge of the grid to ensure their states are valid.
  fn update_edge_cells(&mut self, edge_cell_igs: &mut VecDeque<Point<InternalGrid>>, cg: &Point<ChunkGrid>, i: &mut i32) {
    while let Some(cell_ig) = edge_cell_igs.pop_front() {
      for (connection, _) in get_connection_points(&cell_ig)
        .into_iter()
        .filter(|(_, point)| point.is_outside_grid())
      {
        let no_neighbours_tile = &self.no_neighbours_tile;
        let cell = self
          .object_grid
          .get_mut(cell_ig.y as usize)
          .and_then(|row| row.get_mut(cell_ig.x as usize))
          .expect("Failed to get edge cell coordinates that came from inside the object grid");
        let previous_entropy = cell.get_entropy();
        match cell.reduce(no_neighbours_tile, &connection, false) {
          Ok(true) => {
            trace!(
              "Validating object grid {}: Reduced possible states of {:?} from {:?} to {:?}",
              cg,
              cell.ig,
              previous_entropy,
              cell.get_entropy(),
            );
            *i += 1;
          }
          Ok(false) => {}
          Err(_) => {
            panic!("Validating object grid {}: Failed to reduce edge cell at {:?}", cg, cell.ig,);
          }
        }
      }
    }
  }

  /// Applies a reference cell's constraints directly to its neighbour.
  pub(crate) fn reduce_neighbour(
    &mut self,
    reference_ig: &Point<InternalGrid>,
    neighbour_ig: &Point<InternalGrid>,
    where_is_neighbour_for_reference: &Connection,
    is_failure_log_level_increased: bool,
  ) -> Result<bool, PropagationFailure> {
    let (reference, neighbour) = self
      .get_cells_mut(reference_ig, neighbour_ig)
      .expect("reference and neighbour coordinates are distinct and inside the object grid");
    if neighbour.is_collapsed() {
      neighbour.verify(reference, where_is_neighbour_for_reference, is_failure_log_level_increased)?;
      Ok(false)
    } else {
      neighbour.reduce(reference, where_is_neighbour_for_reference, is_failure_log_level_increased)
    }
  }

  /// Returns two mutable [`Cell`]s which requires a lot of silly stuff, so that we can use `split_at_mut` to prove that
  /// they cannot overlap and therefore make the borrow checker happy.
  fn get_cells_mut(
    &mut self,
    first_ig: &Point<InternalGrid>,
    second_ig: &Point<InternalGrid>,
  ) -> Option<(&mut Cell, &mut Cell)> {
    if first_ig == second_ig || first_ig.is_outside_grid() || second_ig.is_outside_grid() {
      return None;
    }

    let (first_x, first_y) = (first_ig.x as usize, first_ig.y as usize);
    let (second_x, second_y) = (second_ig.x as usize, second_ig.y as usize);
    if first_y == second_y {
      let row = self.object_grid.get_mut(first_y)?;
      return if first_x < second_x {
        let (left, right) = row.split_at_mut(second_x);
        Some((left.get_mut(first_x)?, right.first_mut()?))
      } else {
        let (left, right) = row.split_at_mut(first_x);
        Some((right.first_mut()?, left.get_mut(second_x)?))
      };
    }

    if first_y < second_y {
      let (upper, lower) = self.object_grid.split_at_mut(second_y);
      Some((
        upper.get_mut(first_y)?.get_mut(first_x)?,
        lower.first_mut()?.get_mut(second_x)?,
      ))
    } else {
      let (upper, lower) = self.object_grid.split_at_mut(first_y);
      Some((
        lower.first_mut()?.get_mut(first_x)?,
        upper.get_mut(second_y)?.get_mut(second_x)?,
      ))
    }
  }

  /// Iterates over all object cells.
  pub(crate) fn iter(&self) -> impl Iterator<Item = &Cell> {
    self.object_grid.iter().flatten()
  }

  pub fn get_cell(&self, ig: &Point<InternalGrid>) -> Option<&Cell> {
    if ig.is_outside_grid() {
      return None;
    }
    self.object_grid.get(ig.y as usize).and_then(|row| row.get(ig.x as usize))
  }

  pub fn get_cell_mut(&mut self, ig: &Point<InternalGrid>) -> Option<&mut Cell> {
    if ig.is_outside_grid() {
      return None;
    }
    self
      .object_grid
      .get_mut(ig.y as usize)
      .and_then(|row| row.get_mut(ig.x as usize))
  }

  pub fn calculate_total_entropy(&self) -> i32 {
    self.object_grid.iter().flatten().map(|cell| cell.get_entropy() as i32).sum()
  }

  pub fn snapshot(&self) -> ObjectGridSnapshot {
    ObjectGridSnapshot {
      object_grid: self.object_grid.clone(),
    }
  }

  pub fn restore_from_snapshot(&mut self, snapshot: &ObjectGridSnapshot) {
    self.object_grid = snapshot.object_grid.clone();
  }

  pub const fn is_failure_log_level_increased(&self) -> bool {
    self.is_failure_log_level_increased
  }

  pub const fn increase_failure_log_level(&mut self) {
    self.is_failure_log_level_increased = true;
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  impl ObjectGrid {
    pub fn default_walkable(cg: Point<ChunkGrid>) -> Self {
      let mut grid = Self::default(cg);
      for row in &mut grid.object_grid {
        for cell in row {
          cell.initialise(TerrainType::Land2, TileType::Fill, &[], vec![], false);
        }
      }

      grid
    }
  }

  #[test]
  fn reduce_neighbour_updates_cells_in_every_cardinal_direction() {
    let mut grid = ObjectGrid::default(Point::new_chunk_grid(0, 0));
    let reference_ig = Point::new_internal_grid(1, 1);
    let mut reference = Cell::new(reference_ig.x, reference_ig.y);
    reference.initialise(
      TerrainType::Any,
      TileType::Fill,
      &[TerrainState::new_with_no_neighbours(
        crate::generation::object::lib::ObjectName::Empty,
        0,
        1,
      )],
      vec![],
      false,
    );
    *grid.get_cell_mut(&reference_ig).unwrap() = reference;

    for (connection, target_ig) in get_connection_points(&reference_ig) {
      let mut target = Cell::new(target_ig.x, target_ig.y);
      target.initialise(
        TerrainType::Any,
        TileType::Fill,
        &[
          TerrainState::new_with_no_neighbours(crate::generation::object::lib::ObjectName::Empty, 0, 1),
          TerrainState::new_with_no_neighbours(crate::generation::object::lib::ObjectName::Land1IndividualObject1, 1, 1),
        ],
        vec![],
        false,
      );
      *grid.get_cell_mut(&target_ig).unwrap() = target;

      let has_changed = grid.reduce_neighbour(&reference_ig, &target_ig, &connection, false).unwrap();

      assert!(has_changed);
      assert_eq!(grid.get_cell(&target_ig).unwrap().get_possible_states().len(), 1);
    }
  }

  #[test]
  fn restore_from_snapshot_only_restores_object_cells() {
    let mut grid = ObjectGrid::default(Point::new_chunk_grid(0, 0));
    let snapshot = grid.snapshot();
    let mut cell = Cell::new(2, 2);
    cell.mark_as_collapsed(crate::generation::object::lib::ObjectName::PathTop);
    *grid.get_cell_mut(&Point::new_internal_grid(2, 2)).unwrap() = cell;

    grid.restore_from_snapshot(&snapshot);

    assert!(!grid.get_cell(&Point::new_internal_grid(2, 2)).unwrap().is_collapsed());
  }
}
