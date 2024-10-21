use bevy::app::{App, Plugin};
use bevy::log::{debug, info, warn};
use bevy::prelude::Res;
use rand::prelude::StdRng;
use rand::Rng;

use crate::generation::object::lib::{Cell, NoPossibleStatesFailure, ObjectGrid};
use crate::resources::Settings;

pub struct WfcPlugin;

impl Plugin for WfcPlugin {
  fn build(&self, _app: &mut App) {}
}

pub fn determine_objects_in_grid(mut rng: &mut StdRng, grid: &mut ObjectGrid, _settings: &Res<Settings>) {
  let mut snapshots = vec![];
  let mut wave_count = 0;
  let mut has_entropy = true;
  let mut error_count = 0;

  while has_entropy {
    match iterate(&mut rng, grid) {
      Ok(is_done) => {
        let grid_clone = grid.clone();
        // TODO: Consider keeping snapshots for the last few waves only
        snapshots.push(grid_clone);
        wave_count += 1;
        debug!("Completed [{:?}] grid wave {}", grid.terrain, wave_count);
        has_entropy = !is_done;
      }
      Err(_) => {
        error_count += 1;
        let snapshot_index = snapshots.len() - error_count;
        let error_message = format!("Failed to get snapshot {}", snapshot_index.to_string());
        grid.restore_from_snapshot(snapshots.get(snapshot_index).expect(error_message.as_str()));
        warn!(
          "Failed to reduce entropy in [{:?}] grid during wave {} - restored snapshot {} out of {}",
          grid.terrain,
          wave_count + 1,
          snapshot_index,
          snapshots.len()
        );
      }
    }
  }
}

pub fn iterate(mut rng: &mut StdRng, grid: &mut ObjectGrid) -> Result<bool, NoPossibleStatesFailure> {
  // Observation: Get the cells with the lowest entropy
  let lowest_entropy_cells = grid.get_cells_with_lowest_entropy();
  if lowest_entropy_cells.is_empty() {
    info!("No more cells to collapse in this [{:?}] grid", grid.terrain);
    return Ok(true);
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
    let mut neighbours = grid.get_neighbours(&cell.cg);
    for (connection, neighbour) in neighbours.iter_mut() {
      if !neighbour.is_collapsed {
        if let Ok((has_changed, neighbour_cell)) = neighbour.clone_and_reduce(&cell, &connection) {
          if has_changed {
            stack.push(neighbour_cell);
          }
        } else {
          return Err(NoPossibleStatesFailure {});
        }
      } else {
        if let Err(NoPossibleStatesFailure {}) = neighbour.verify(&cell, &connection) {
          return Err(NoPossibleStatesFailure {});
        }
      }
    }
  }

  Ok(false)
}
