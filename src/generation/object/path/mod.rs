use crate::coords::Point;
use crate::coords::point::InternalGrid;
use crate::generation::lib::Direction;
use crate::generation::object::lib::{CellRef, ObjectGrid, ObjectName};
use crate::generation::resources::Metadata;
use crate::resources::Settings;
use bevy::app::{App, Plugin};
use bevy::log::*;
use rand::Rng;
use rand::prelude::StdRng;
use std::sync::Arc;

pub struct PathGenerationPlugin;

impl Plugin for PathGenerationPlugin {
  fn build(&self, _: &mut App) {}
}

pub fn calculate_paths(
  settings: &Settings,
  metadata: &Metadata,
  mut object_grid: ObjectGrid,
  mut rng: StdRng,
) -> ObjectGrid {
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
  if connection_points.len() == 1 {
    warn!(
      "Skipping path generation for chunk {} because it has only 1 connection point",
      cg
    );
    return object_grid;
  }
  debug!(
    "Generating path for chunk {} which has [{}] connection points: {}",
    cg,
    connection_points.len(),
    connection_points
      .iter()
      .map(|p| format!("{}", p))
      .collect::<Vec<_>>()
      .join(", ")
  );

  // Randomly select the initial start point and remove it from remaining points
  let mut remaining_points = connection_points.clone();
  let start_index = rng.random_range(..remaining_points.len());
  let mut current_start = remaining_points.remove(start_index);
  let mut total_path_cells = 0;

  // Loop through remaining connection points, always picking the closest one
  while !remaining_points.is_empty() {
    // Identify the start and target points for the path segment
    let closest_index = remaining_points
      .iter()
      .enumerate()
      .min_by(|(_, a), (_, b)| {
        current_start
          .distance_to(a)
          .partial_cmp(&current_start.distance_to(b))
          .expect("Failed to compare distances")
      })
      .map(|(index, _)| index)
      .expect("Failed to find closest point");
    let target_point = remaining_points.remove(closest_index);
    let start_cell = object_grid.get_cell_ref(&current_start).expect("Failed to get start cell");
    let target_cell = object_grid.get_cell_ref(&target_point).expect("Failed to get target cell");

  // Run the pathfinding algorithm and collapse cells on the path
  let path = run_algorithm(start_cell, target_cell);
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
    // Run the pathfinding algorithm and collapse cells on the path
    debug!(
      "Generating path segment for chunk {} from {:?} to {:?}",
      cg, current_start, target_point
    );
    let path = run_algorithm(start_cell, target_cell);
    for (i, (point, direction)) in path.iter().enumerate() {
      let prev_direction = if i > 0 { Some(path[i - 1].1.to_opposite()) } else { None };
      let object_name = determine_path_object_name(prev_direction.as_ref(), Some(direction));
      trace!(
        "Path segment [{}/{}] at point {:?} with next cell [{:?}] and previous cell [{:?}] has name [{:?}]",
        i + 1,
        path.len(),
        point,
        direction,
        prev_direction,
        object_name
      );
      let cell = object_grid
        .get_cell_mut(&point)
        .expect(format!("Failed to get cell at point {:?}", point).as_str());
      cell.pre_collapse(object_name);
    }
    object_grid.reinitialise();
    debug!(
      "Generated path segment for chunk {} from {:?} to {:?} with [{}] cells: {}",
      cg,
      current_start,
      target_point,
      path.len(),
      path.iter().map(|p| format!("{:?}", p)).collect::<Vec<_>>().join(", ")
    );

    // Set the target as the new start for the next iteration
    total_path_cells += path.len();
    break;
    // current_start = target_point;
  }

  debug!(
    "Generated complete path network for chunk {} with [{}] total cells across all segments",
    cg, total_path_cells
  );

  object_grid
}

pub fn run_algorithm(start_cell: &CellRef, target_cell: &CellRef) -> Vec<(Point<InternalGrid>, Direction)> {
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
        "✅  Arrived at target cell {:?}, now reconstructing the path",
        current_cell.try_lock().expect("Failed to lock current cell").ig
      );
      let mut path = Vec::new();
      let mut cell = Some(current_cell.clone());

      while let Some(current) = cell {
        let (current_ig, next_cell) = {
          let cell = current.try_lock().expect("Failed to lock current cell");

          (cell.ig, cell.get_connection().as_ref().cloned())
        };
        let direction_to_next = if let Some(ref next_cell) = next_cell {
          let next_cell_ig = next_cell.try_lock().expect("Failed to lock next cell").ig;
          Direction::from_points(&current_ig, &next_cell_ig)
        } else {
          Direction::Center
        };
        path.push((current_ig, direction_to_next));
        cell = next_cell;
      }

      return path;
    }

    // If we haven't reached the target, process the current cell's neighbours
    let (current_g, current_ig, current_neighbours) = {
      let c = current_cell.lock().expect("Failed to lock current cell");
      (c.get_g(), c.ig, c.get_neighbours().clone())
    };
    let target_ig = get_target_cell_ig(target_cell);

    trace!("Processing cell at {:?}", current_ig);
    for neighbour in current_neighbours {
      let mut n = neighbour.lock().expect("Failed to lock neighbour");

      // Skip if the neighbour has already been processed
      if processed.iter().any(|c| Arc::ptr_eq(c, &neighbour)) {
        trace!(" └─> Skipping neighbour {:?} because it has already been processed", n.ig);
        continue;
      }

      // If the neighbour is not in the cells to search or if the G cost to the neighbour is
      // lower than its current G cost...
      let is_not_in_cells_to_search = !to_search.iter().any(|n_ref| Arc::ptr_eq(n_ref, &neighbour));
      let g_cost_to_neighbour = current_g + calculate_distance_cost(&current_ig, &n.ig);

      if is_not_in_cells_to_search || g_cost_to_neighbour < n.get_g() {
        // ...then update the neighbour's G cost, and set the current cell as its connection
        n.set_g(g_cost_to_neighbour);
        n.set_connection(&current_cell);
        let distance_cost = calculate_distance_cost(&n.get_ig(), &target_ig);

        // ...and set the neighbour's H cost to the distance to the target cell,
        // if it is not already in the cells to search
        if is_not_in_cells_to_search {
          n.set_h(distance_cost);
          to_search.push(neighbour.clone());
        }

        trace!(
          " └─> Set as connection of {}, update {} G to [{}]{}",
          n.get_ig(),
          n.get_ig(),
          g_cost_to_neighbour,
          if is_not_in_cells_to_search {
            "".to_string()
          } else {
            format!(", H to [{}], plus adding it to cell to search", &distance_cost)
          }
        );
      }
    }
  }

  vec![]
}

fn get_target_cell_ig(target_cell: &CellRef) -> Point<InternalGrid> {
  target_cell.lock().expect("Failed to lock target cell").get_ig().clone()
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

fn determine_path_object_name(
  previous_cell_direction: Option<&Direction>,
  next_cell_direction: Option<&Direction>,
) -> ObjectName {
  use Direction::*;
  let result = match (previous_cell_direction, next_cell_direction) {
    (Some(Top), Some(Right)) | (Some(Right), Some(Top)) => ObjectName::PathTopRight,
    (Some(Top), Some(Bottom)) | (Some(Bottom), Some(Top)) => ObjectName::PathVertical,
    (Some(Right), Some(Left)) | (Some(Left), Some(Right)) => ObjectName::PathHorizontal,
    (Some(Bottom), Some(Left)) | (Some(Left), Some(Bottom)) => ObjectName::PathBottomLeft,
    (Some(Bottom), Some(Right)) | (Some(Right), Some(Bottom)) => ObjectName::PathBottomRight,
    (Some(Top), Some(Left)) | (Some(Left), Some(Top)) => ObjectName::PathTopLeft,
    (Some(Top), None) | (Some(Top), Some(Center)) | (None, Some(Top)) => ObjectName::PathTop,
    (Some(Right), None) | (Some(Right), Some(Center)) | (None, Some(Right)) => ObjectName::PathRight,
    (Some(Bottom), None) | (Some(Bottom), Some(Center)) | (None, Some(Bottom)) => ObjectName::PathBottom,
    (Some(Left), None) | (Some(Left), Some(Center)) | (None, Some(Left)) => ObjectName::PathLeft,
    _ => ObjectName::PathUndefined,
  };

  result
}
