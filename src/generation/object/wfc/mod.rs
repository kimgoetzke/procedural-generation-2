use crate::generation::lib::shared;
use crate::generation::object::lib::{Cell, IterationResult, ObjectData, ObjectGrid, TileData};
use crate::resources::Settings;
use bevy::app::{App, Plugin};
use bevy::log::*;
use rand::Rng;
use rand::prelude::StdRng;

pub struct WfcPlugin;

impl Plugin for WfcPlugin {
  fn build(&self, _app: &mut App) {}
}

/// The entry point for running the wave function collapse algorithm to determine the object sprites in the grid.
pub fn determine_objects_in_grid(
  mut rng: &mut StdRng,
  object_generation_data: &mut (ObjectGrid, Vec<TileData>),
  settings: &Settings,
) -> Vec<ObjectData> {
  let start_time = shared::get_time();
  let is_decoration_enabled = settings.object.generate_decoration;
  let (mut snapshot_error_count, mut iter_error_count, mut total_error_count) = (0, 0, 0);
  if is_decoration_enabled {
    let grid = &mut object_generation_data.0;
    let mut snapshots = vec![];
    let mut iter_count = 1;
    let mut has_entropy = true;

    while has_entropy {
      match iterate(&mut rng, grid) {
        IterationResult::Failure => handle_failure(
          grid,
          &mut snapshots,
          &mut iter_count,
          &mut snapshot_error_count,
          &mut iter_error_count,
          &mut total_error_count,
        ),
        result => handle_success(
          grid,
          &mut snapshots,
          &mut iter_count,
          &mut has_entropy,
          &mut iter_error_count,
          result,
        ),
      }
    }
  } else {
    debug!(
      "Skipped decoration generation for {} because it is disabled",
      object_generation_data.0.cg
    );
  }

  let object_data = create_object_data(&object_generation_data.0, &object_generation_data.1, is_decoration_enabled);
  log_summary(
    start_time,
    snapshot_error_count,
    total_error_count,
    &object_generation_data.0,
    is_decoration_enabled,
  );

  object_data
}

/// A single iteration over the object grid that performs the following steps:
/// 1. **Observation**: Get the cells with the lowest entropy.
/// 2. **Collapse**: Collapse a random cell from the cells with the lowest entropy.
/// 3. **Propagation**: Update every neighbour's states and the grid, if possible.
///
/// This method is the central part of the wave function collapse algorithm and is called repeatedly until no more
/// cells can be collapsed.
fn iterate(mut rng: &mut StdRng, grid: &mut ObjectGrid) -> IterationResult {
  // Observation: Get the cells with the lowest entropy
  let lowest_entropy_cells = grid.get_cells_with_lowest_entropy();
  if lowest_entropy_cells.is_empty() {
    trace!("No more cells to collapse in object grid {}", grid.cg);
    return IterationResult::Ok;
  }

  // Collapse: Collapse random cell from the cells with the lowest entropy
  let index = rng.random_range(0..lowest_entropy_cells.len());
  let random_cell: &Cell = lowest_entropy_cells
    .get(index)
    .expect(format!("Failed to get random cell during processing of object grid {}", grid.cg).as_str());
  let mut random_cell_clone = random_cell.clone();
  random_cell_clone.collapse(&mut rng);

  // Propagation: Update every neighbours' states and the grid
  let mut stack: Vec<Cell> = vec![random_cell_clone];
  while let Some(cell) = stack.pop() {
    grid.set_cell(cell.clone());
    for (connection, neighbour) in grid.get_neighbours(&cell).iter_mut() {
      if !neighbour.is_collapsed() {
        if let Ok((has_changed, neighbour_cell)) = neighbour.clone_and_reduce(&cell, &connection) {
          if has_changed {
            stack.push(neighbour_cell);
          }
        } else {
          return IterationResult::Failure;
        }
      } else {
        if let Err(_) = neighbour.verify(&cell, &connection) {
          return IterationResult::Failure;
        }
      }
    }
  }

  IterationResult::Incomplete
}

fn handle_failure(
  grid: &mut ObjectGrid,
  snapshots: &mut Vec<ObjectGrid>,
  iter_count: &mut i32,
  snapshot_error_count: &mut usize,
  iter_error_count: &mut usize,
  total_error_count: &mut i32,
) {
  *iter_error_count += 1;
  *total_error_count += 1;
  let snapshot_index = snapshots.len().saturating_sub(*iter_error_count);
  let snapshot = snapshots.get(snapshot_index);
  if let Some(snapshot) = snapshot {
    grid.restore_from_snapshot(snapshot);
    log_failure(grid, &snapshots, iter_count, iter_error_count, snapshot_index);
  } else {
    error!(
      "Failed (#{}) to reduce entropy in object grid {} during iteration {} - no snapshot available",
      iter_error_count, grid.cg, iter_count
    );
    *snapshot_error_count += 1;
  }
  snapshots.truncate(snapshot_index);
}

fn handle_success(
  grid: &mut ObjectGrid,
  snapshots: &mut Vec<ObjectGrid>,
  iter_count: &mut i32,
  has_entropy: &mut bool,
  iter_error_count: &mut usize,
  result: IterationResult,
) {
  let current_entropy = grid.calculate_total_entropy();
  log_completion(grid, iter_count, iter_error_count, current_entropy);
  if *iter_count % 10 == 0 {
    snapshots.push(grid.clone());
  }
  *has_entropy = result == IterationResult::Incomplete;
  *iter_count += 1;
  *iter_error_count = 0;
}

fn create_object_data(grid: &ObjectGrid, tile_data: &Vec<TileData>, is_decoration_enabled: bool) -> Vec<ObjectData> {
  let mut object_data = vec![];
  object_data.extend(
    tile_data
      .iter()
      .filter_map(|tile_data| {
        if is_decoration_enabled {
          grid
            .get_cell(&tile_data.flat_tile.coords.internal_grid)
            .filter(|cell| cell.get_index() != 0) // Sprite index 0 is always transparent
            .map(|cell| ObjectData::from(cell, tile_data))
        } else {
          grid
            .get_cell(&tile_data.flat_tile.coords.internal_grid)
            .filter(|cell| cell.get_index() != 0) // Sprite index 0 is always transparent
            .filter(|cell| cell.is_collapsed()) // Ignore non-collapsed cells since WFC did not run
            .map(|cell| ObjectData::from(cell, tile_data))
        }
      })
      .collect::<Vec<ObjectData>>(),
  );
  object_data
}

fn log_completion(grid: &mut ObjectGrid, iter_count: &i32, iter_error_count: &mut usize, current_entropy: i32) {
  trace!(
    "Completed object grid {} iteration {} (encountering {} errors) and with a total entropy of {}",
    grid.cg, iter_count, iter_error_count, current_entropy
  );
}

fn log_failure(
  grid: &mut ObjectGrid,
  snapshots: &Vec<ObjectGrid>,
  iteration_count: &i32,
  iteration_error_count: &usize,
  snapshot_index: usize,
) {
  trace!(
    "Failed (#{}) to reduce entropy in object grid {} during iteration {} - restored snapshot {} out of {}",
    iteration_error_count,
    grid.cg,
    iteration_count,
    snapshot_index,
    snapshots.len()
  );
}

fn log_summary(
  start_time: u128,
  snapshot_error_count: usize,
  total_error_count: i32,
  grid: &ObjectGrid,
  is_decoration_enabled: bool,
) {
  match (is_decoration_enabled, total_error_count, snapshot_error_count) {
    (false, _, _) => {
      trace!(
        "Completed converting object grid to object data for {} in {} ms on {}",
        grid.cg,
        shared::get_time() - start_time,
        shared::thread_name()
      );
    }
    (true, 0, 0) => {
      trace!(
        "Completed wave function collapse for {} in {} ms on {}",
        grid.cg,
        shared::get_time() - start_time,
        shared::thread_name()
      );
    }
    (true, 1..15, 0) => {
      debug!(
        "Completed wave function collapse for {} (resolving {} errors) in {} ms on {}",
        grid.cg,
        total_error_count,
        shared::get_time() - start_time,
        shared::thread_name()
      );
    }
    (true, 15.., 0) => {
      warn!(
        "Completed wave function collapse for {} (resolving {} errors) in {} ms on {}",
        grid.cg,
        total_error_count,
        shared::get_time() - start_time,
        shared::thread_name()
      );
    }
    _ => {
      error!(
        "Completed wave function collapse for {} (resolving {} errors and leaving {} unresolved) in {} ms on {}",
        grid.cg,
        total_error_count,
        snapshot_error_count,
        shared::get_time() - start_time,
        shared::thread_name()
      );
    }
  }
}
