use crate::coords::Point;
use crate::coords::point::InternalGrid;
use crate::generation::object::lib::{CellRef, ObjectGrid};
use crate::generation::resources::Metadata;
use crate::resources::Settings;
use bevy::app::{App, Plugin};
use bevy::log::*;
use std::sync::Arc;

pub struct PathGenerationPlugin;

impl Plugin for PathGenerationPlugin {
  fn build(&self, _: &mut App) {}
}

pub fn calculate_paths(settings: &Settings, metadata: &Metadata, mut object_grid: ObjectGrid) -> ObjectGrid {
  let cg = object_grid.cg;
  if !settings.object.generate_paths {
    debug!("Skipped path generation for {} because it is disabled", cg);
    return object_grid;
  }
  let connection_points = metadata
    .connection_points
    .get(&cg)
    .expect(format!("Failed to get connection points for {}", cg).as_str());

  // TODO: Extend implementation to handle 1 or more than 2 connection points
  if connection_points.is_empty() {
    debug!(
      "Skipping path generation for chunk {} because it has no connection points",
      cg
    );
    return object_grid;
  }
  if connection_points.len() != 2 {
    warn!(
      "Skipping path generation for chunk {} because MVP implementation is only for exactly 2 connection points but there are [{}]",
      cg,
      connection_points.len()
    );
    return object_grid;
  }
  trace!(
    "Generating path for chunk {} which has [{}] connection points: {}",
    cg,
    connection_points.len(),
    connection_points
      .iter()
      .map(|p| format!("{}", p))
      .collect::<Vec<_>>()
      .join(", ")
  );

  // Get start and target cells based on connection points
  let start_node = object_grid
    .get_cell_ref(&connection_points[0])
    .expect("Failed to get start node");
  let target_node = object_grid
    .get_cell_ref(&connection_points[1])
    .expect("Failed to get target node");

  // Run the pathfinding algorithm and TODO: Collapse or mark cells on the path
  let path = run_algorithm(start_node, target_node);
  for point in &path {
    let cell = object_grid
      .get_cell_mut(point)
      .expect(format!("Failed to get cell at point {:?}", point).as_str());
    cell.pre_collapse(8);
  }

  debug!(
    "Generated path for chunk {} with [{}] nodes in the path: {:?}",
    cg,
    path.len(),
    path
  );

  object_grid
}

pub fn run_algorithm(start_cell: &CellRef, target_cell: &CellRef) -> Vec<Point<InternalGrid>> {
  let mut to_search: Vec<CellRef> = vec![start_cell.clone()];
  let mut processed: Vec<CellRef> = Vec::new();

  while !to_search.is_empty() {
    // Find the cell with the lowest F cost, using H cost as a tiebreaker
    let mut current_cell = to_search[0].clone();
    for cell in &to_search {
      if Arc::ptr_eq(&current_cell, cell) {
        continue;
      }
      let cell_guard = cell.lock().expect("Failed to lock cell to search");
      let current_guard = current_cell.lock().expect("Failed to lock current cell");
      let cell_f = cell_guard.get_f();
      let cell_h = cell_guard.get_h();
      let current_f = current_guard.get_f();
      let current_h = current_guard.get_h();
      drop(current_guard);
      drop(cell_guard);
      if cell_f < current_f || (cell_f == current_f && cell_h < current_h) {
        current_cell = cell.clone();
      }
    }

    // Mark this cell with the lowest F cost as processed and remove it from the cells to search
    processed.push(current_cell.clone());
    to_search.retain(|cell| !Arc::ptr_eq(cell, &current_cell));

    // If we have reached the target cell, reconstruct the path and return it
    if Arc::ptr_eq(&current_cell, &target_cell) {
      trace!(
        "✅  Arrived at target node {:?}, now reconstructing the path",
        current_cell.try_lock().expect("Failed to lock current node").ig
      );
      let mut path = Vec::new();
      let mut cell = Some(current_cell.clone());

      while let Some(current) = cell {
        let (current_ig, next_cell) = {
          let cell = current.lock().expect("Failed to lock current node");

          (cell.ig, cell.get_connection().as_ref().cloned())
        };
        path.push(current_ig);
        cell = next_cell;
      }

      path.reverse();
      return path;
    }

    // If we haven't reached the target, process the current cell's neighbours
    let (current_g, current_ig, current_neighbours) = {
      let c = current_cell.lock().expect("Failed to lock current node");
      (c.get_g(), c.ig, c.get_neighbours().clone())
    };
    let target_ig = get_target_cell_ig(target_cell);

    trace!("Processing node at {:?}", current_ig);
    for neighbour in current_neighbours {
      let mut n = neighbour.lock().expect("Failed to lock neighbour");

      // Skip if the neighbour has already been processed
      if processed.iter().any(|c| Arc::ptr_eq(c, &neighbour)) {
        trace!(" └─> Skipping neighbour {:?} because it has already been processed", n.ig);
        continue;
      }

      // If the neighbour is not in the cells to search or if the G cost to the neighbour is
      // lower than its current G cost...
      let is_not_in_nodes_to_search = !to_search.iter().any(|n_ref| Arc::ptr_eq(n_ref, &neighbour));
      let g_cost_to_neighbour = current_g + calculate_distance_cost(&current_ig, &n.ig);

      if is_not_in_nodes_to_search || g_cost_to_neighbour < n.get_g() {
        // ...then update the neighbour's G cost, and set the current cell as its connection
        n.set_g(g_cost_to_neighbour);
        n.set_connection(&current_cell);
        let distance_cost = calculate_distance_cost(&n.get_ig(), &target_ig);

        // ...and set the neighbour's H cost to the distance to the target cell,
        // if it is not already in the cells to search
        if is_not_in_nodes_to_search {
          n.set_h(distance_cost);
          to_search.push(neighbour.clone());
        }

        trace!(
          " └─> Set as connection of {}, update {} G to [{}]{}",
          n.get_ig(),
          n.get_ig(),
          g_cost_to_neighbour,
          if is_not_in_nodes_to_search {
            "".to_string()
          } else {
            format!(", H to [{}], plus adding it to nodes to search", &distance_cost)
          }
        );
      }
    }
  }

  vec![]
}

fn get_target_cell_ig(target_cell: &CellRef) -> Point<InternalGrid> {
  target_cell.lock().expect("Failed to lock target node").get_ig().clone()
}

/// Calculates the costs based on the distance between two points in the internal grid, adjusting the cost based on the
/// direction of movement.
/// - If the movement is diagonal, the cost is `14` per tile moved.
/// - If the movement is horizontal or vertical, the cost is `10` per tile moved.
fn calculate_distance_cost(a: &Point<InternalGrid>, b: &Point<InternalGrid>) -> f32 {
  let x_diff = (a.x - b.x).abs() as f32;
  let y_diff = (a.y - b.y).abs() as f32;

  if x_diff > y_diff {
    14. * y_diff + 10. * (x_diff - y_diff)
  } else {
    14. * x_diff + 10. * (y_diff - x_diff)
  }
}
