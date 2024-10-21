use crate::generation::get_time;
use crate::generation::object::lib::{Cell, IterationResult, ObjectGrid};
use crate::resources::Settings;
use bevy::app::{App, Plugin};
use bevy::log::*;
use bevy::prelude::Res;
use rand::prelude::StdRng;
use rand::Rng;

pub struct WfcPlugin;

impl Plugin for WfcPlugin {
  fn build(&self, _app: &mut App) {}
}

pub fn determine_objects_in_grid(mut rng: &mut StdRng, grid: &mut ObjectGrid, _settings: &Res<Settings>) {
  let start_time = get_time();
  let mut snapshots = vec![];
  let mut iteration_count = 1;
  let mut has_entropy = true;
  let mut iteration_error_count: usize = 0;
  let mut total_error_count = 0;
  let mut last_entropy = grid.calculate_total_entropy();

  while has_entropy {
    match iterate(&mut rng, grid) {
      IterationResult::Failure => {
        iteration_error_count += 1;
        total_error_count += 1;
        let snapshot_index = snapshots.len() - (iteration_error_count * 2);
        grid.restore_from_snapshot(
          snapshots
            .get(snapshot_index)
            .expect(format!("Failed to get snapshot {}", snapshot_index.to_string()).as_str()),
        );
        warn!(
          "Failed (#{}) to reduce entropy in object grid during iteration {} - restored snapshot {} out of {}",
          iteration_error_count,
          iteration_count,
          snapshot_index,
          snapshots.len()
        );
        snapshots.truncate(snapshot_index + 1);
        iteration_count += 1;
      }
      result => {
        let current_entropy = grid.calculate_total_entropy();
        debug!(
          "Completed object grid iteration {} after resolving {} errors and with a total entropy of {}",
          iteration_count, iteration_error_count, current_entropy
        );
        if last_entropy - current_entropy > 0 {
          snapshots.push(grid.clone());
          last_entropy = current_entropy;
          debug!(
            "Snapshot taken at iteration {} (entropy={}) - snapshots: {}",
            iteration_count,
            current_entropy,
            snapshots.len()
          );
        }
        has_entropy = result == IterationResult::Incomplete;
        iteration_count += 1;
        iteration_error_count = 0;
      }
    }
  }

  debug!(
    "Completed determining objects - and resolved {} errors - in {} ms",
    total_error_count,
    get_time() - start_time
  );
}

pub fn iterate(mut rng: &mut StdRng, grid: &mut ObjectGrid) -> IterationResult {
  // Observation: Get the cells with the lowest entropy
  let lowest_entropy_cells = grid.get_cells_with_lowest_entropy();
  if lowest_entropy_cells.is_empty() {
    info!("No more cells to collapse in object grid");
    return IterationResult::Ok;
  }

  // Collapse: Collapse random cell from the cells with the lowest entropy
  let index = rng.gen_range(0..lowest_entropy_cells.len());
  let random_cell: &Cell = lowest_entropy_cells.get(index).expect("Failed to get random cell");
  let mut random_cell_clone = random_cell.clone();
  random_cell_clone.collapse(&mut rng);

  // Propagation: Update every neighbours' states and the grid
  let mut stack: Vec<Cell> = vec![random_cell_clone];
  while let Some(cell) = stack.pop() {
    grid.set_cell(cell.clone());
    for (connection, neighbour) in grid.get_neighbours(&cell.cg).iter_mut() {
      if !neighbour.is_collapsed {
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
