mod cell_entropy_index;

use crate::constants::{CHUNK_SIZE, WAVE_FUNCTION_COLLAPSE_SNAPSHOT_INTERVAL, WAVE_FUNCTION_COLLAPSE_WARNING_FREQUENCY};
use crate::coords::Point;
use crate::coords::point::InternalGrid;
use crate::generation::lib::shared;
use crate::generation::object::lib::{IterationResult, ObjectGrid, ObjectGridSnapshot, get_connection_points};
use crate::generation::object::wfc::cell_entropy_index::CellEntropyIndex;
use crate::resources::Settings;
use bevy::app::{App, Plugin};
use bevy::log::*;
use rand::prelude::StdRng;

/// Contains the main logic for the wave function collapse algorithm used to determine decorative objects in the grid.
pub struct WfcPlugin;

impl Plugin for WfcPlugin {
  fn build(&self, _app: &mut App) {}
}

/// The entry point for running the wave function collapse algorithm to determine the object sprites in the grid.
pub fn place_decorative_objects_on_grid(object_grid: &mut ObjectGrid, settings: &Settings, rng: &mut StdRng) {
  let start_time = shared::get_time();
  let mut next_warning_time = start_time + WAVE_FUNCTION_COLLAPSE_WARNING_FREQUENCY;
  object_grid.validate();
  let (mut snapshot_error_count, mut iter_error_count, mut total_error_count) = (0, 0, 0);
  let is_decoration_enabled = settings.object.generate_decoration;
  if is_decoration_enabled {
    let mut snapshots = vec![];
    let mut iter_count = 1;
    let mut has_entropy = true;
    let mut entropy_index = CellEntropyIndex::from_cells(object_grid.iter());
    let mut propagation_stack = Vec::with_capacity((CHUNK_SIZE * CHUNK_SIZE) as usize);

    while has_entropy {
      match iterate(rng, object_grid, &mut entropy_index, &mut propagation_stack) {
        IterationResult::Failure => {
          handle_failure(
            object_grid,
            &mut snapshots,
            &mut iter_count,
            &mut snapshot_error_count,
            &mut iter_error_count,
            &mut total_error_count,
            &mut next_warning_time,
            start_time,
          );
          entropy_index.rebuild(object_grid.iter());
        }
        result => handle_success(
          object_grid,
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
      "Skipped placing decorative objects for {} because it is disabled",
      object_grid.cg
    );
  }

  log_summary(
    start_time,
    snapshot_error_count,
    total_error_count,
    object_grid,
    is_decoration_enabled,
  );
}

/// A single iteration over the object grid that performs the following steps:
/// 1. **Observation**: Get the cells with the lowest entropy.
/// 2. **Collapse**: Collapse a random cell from the cells with the lowest entropy.
/// 3. **Propagation**: Update every neighbour's states and the grid, if possible.
///
/// This method is the central part of the wave function collapse algorithm and is called repeatedly until no more
/// cells can be collapsed.
fn iterate(
  rng: &mut StdRng,
  grid: &mut ObjectGrid,
  entropy_index: &mut CellEntropyIndex,
  propagation_stack: &mut Vec<Point<InternalGrid>>,
) -> IterationResult {
  // Observation: Get a cell with the lowest entropy
  let Some(ig) = entropy_index.choose_lowest_entropy_cell(rng) else {
    trace!("No more cells to collapse in object grid {}", grid.cg);
    return IterationResult::Ok;
  };

  // Collapse: Collapse random cell from the cells with the lowest entropy
  let cg = grid.cg;
  let random_cell = grid
    .get_cell_mut(&ig)
    .unwrap_or_else(|| panic!("Failed to get cell {ig} from object grid {cg} during processing"));
  random_cell.collapse(rng);
  entropy_index.update(random_cell);

  // Propagation: Update every neighbour's states in place
  propagation_stack.clear();
  propagation_stack.push(ig);
  let is_failure_log_level_increased = grid.is_failure_log_level_increased();
  while let Some(reference_ig) = propagation_stack.pop() {
    for (connection, neighbour_ig) in get_connection_points(&reference_ig) {
      if neighbour_ig.is_outside_grid() {
        continue;
      }
      match grid.reduce_neighbour(&reference_ig, &neighbour_ig, &connection, is_failure_log_level_increased) {
        Ok(true) => {
          let neighbour = grid
            .get_cell(&neighbour_ig)
            .expect("neighbour coordinates came from inside the object grid");
          entropy_index.update(neighbour);
          propagation_stack.push(neighbour_ig);
        }
        Ok(false) => {}
        Err(_) => return IterationResult::Failure,
      }
    }
  }

  IterationResult::Incomplete
}

fn handle_failure(
  grid: &mut ObjectGrid,
  snapshots: &mut Vec<ObjectGridSnapshot>,
  iter_count: &mut i32,
  snapshot_error_count: &mut usize,
  iter_error_count: &mut usize,
  total_error_count: &mut i32,
  next_warning_time: &mut u128,
  start_time: u128,
) {
  *iter_error_count += 1;
  *total_error_count += 1;
  let snapshot_index = snapshots.len().saturating_sub(*iter_error_count);
  let snapshot = snapshots.get(snapshot_index);
  if let Some(snapshot) = snapshot {
    grid.restore_from_snapshot(snapshot);
    let now = shared::get_time();
    increase_logging_or_short_circuit(&now, next_warning_time, grid, iter_count, iter_error_count, snapshots);
    log_failure(
      grid,
      snapshots,
      iter_count,
      iter_error_count,
      snapshot_index,
      start_time,
      now,
      next_warning_time,
    );
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
  snapshots: &mut Vec<ObjectGridSnapshot>,
  iter_count: &mut i32,
  has_entropy: &mut bool,
  iter_error_count: &mut usize,
  result: IterationResult,
) {
  log_completion(grid, iter_count, iter_error_count);
  if *iter_count % WAVE_FUNCTION_COLLAPSE_SNAPSHOT_INTERVAL == 0 {
    snapshots.push(grid.snapshot());
  }
  *has_entropy = result == IterationResult::Incomplete;
  *iter_count += 1;
  *iter_error_count = 0;
}

fn log_completion(grid: &ObjectGrid, iter_count: &i32, iter_error_count: &mut usize) {
  trace!(
    "Completed object grid {} iteration {} (encountering {} errors) and with a total entropy of {}",
    grid.cg,
    iter_count,
    iter_error_count,
    grid.calculate_total_entropy()
  );
}

/// This function will initially increase the logging level for failures on the grid for debugging purposes. If the log
/// level has already been increased and there is 1) a low error count and 2) a high iteration count, it will
/// short-circuit the wave function collapse by clearing all snapshots. This is a last resort to avoid infinite loops.
/// If this happens, it may indicate that the implementation of the wave function collapse algorithm is flawed or that
/// the constraints or rulesets provided to the algorithm are unsolvable. However, it can also mean that you simply got
/// unlucky and the randomly selected state to which a cell was collapsed to kept selecting unsolvable states over and
/// over and over.
fn increase_logging_or_short_circuit(
  now: &u128,
  next_warning_time: &mut u128,
  grid: &mut ObjectGrid,
  iter_count: &mut i32,
  iter_error_count: &mut usize,
  snapshots: &mut Vec<ObjectGridSnapshot>,
) {
  if grid.is_failure_log_level_increased() && *iter_error_count < 5 && *iter_count > 40_000 {
    warn!(
      "Attempting to short-circuiting wave function collapse for {} after {} iterations and {} error(s) by clearing all snapshots",
      grid.cg, iter_count, iter_error_count
    );
    snapshots.clear();
    return;
  }
  if now >= next_warning_time {
    grid.increase_failure_log_level();
  }
}

fn log_failure(
  grid: &mut ObjectGrid,
  snapshots: &[ObjectGridSnapshot],
  iteration_count: &i32,
  iteration_error_count: &usize,
  snapshot_index: usize,
  start_time: u128,
  now: u128,
  next_warning_time: &mut u128,
) {
  trace!(
    "Failed (#{}) to reduce entropy in object grid {} during iteration {} - restored snapshot {} out of {}",
    iteration_error_count,
    grid.cg,
    iteration_count,
    snapshot_index,
    snapshots.len()
  );
  if now >= *next_warning_time {
    let elapsed = (now - start_time) / 1_000;
    warn!(
      "Wave function collapse for {} has been running for {} seconds (iteration #{}, error count #{}, {} snapshots)...",
      grid.cg,
      elapsed,
      iteration_count,
      iteration_error_count,
      snapshots.len()
    );
    *next_warning_time = now + WAVE_FUNCTION_COLLAPSE_WARNING_FREQUENCY;
  }
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
