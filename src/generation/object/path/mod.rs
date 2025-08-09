use crate::constants::CHUNK_SIZE;
use crate::coords::Point;
use crate::coords::point::InternalGrid;
use crate::generation::lib::{Direction, get_cardinal_direction_points};
use crate::generation::object::lib::{CellRef, ObjectGrid, ObjectName};
use crate::generation::resources::Metadata;
use crate::resources::Settings;
use bevy::app::{App, Plugin};
use bevy::log::*;
use bevy::platform::collections::HashSet;
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

  // TODO: Check if connection point is_walkable and if not, skip it

  if connection_points.is_empty() {
    debug!(
      "Skipping path generation for chunk {} because it has no connection points",
      cg
    );
    return object_grid;
  } else if connection_points.len() == 1 {
    error!(
      "Skipping path generation for chunk {} because it has only 1 connection point which is a bug",
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
  let mut path: Vec<(Point<InternalGrid>, Direction)> = Vec::new();

  // Loop through remaining connection points, always picking the closest one, and run the algorithm
  while !remaining_points.is_empty() {
    // Make sure the path grid is populated and each cell's neighbours are set
    object_grid.initialise_path_grid();

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

    // Run the pathfinding algorithm
    debug!(
      "Generating path segment for chunk {} from {:?} to {:?}",
      cg, current_start, target_point
    );
    // TODO: Consider using this for all path segments to determine object names to be fixed
    let path_segment: Vec<(Point<InternalGrid>, Direction)> = run_algorithm(start_cell, target_cell);
    path.extend(&path_segment);

    // Collapse the cells along the path
    for (i, (point, next_direction)) in path_segment.iter().enumerate() {
      let prev_direction = if i > 0 {
        path_segment[i - 1].1.to_opposite()
      } else {
        Direction::Center
      };
      let object_name = determine_path_object_name(&prev_direction, next_direction, point);
      trace!(
        "- Path cell [{}/{}] at point {:?} with next cell [{:?}] + previous cell [{:?}] has name [{:?}]",
        i + 1,
        path_segment.len(),
        point,
        next_direction,
        prev_direction,
        object_name
      );
      let cell = object_grid
        .get_cell_mut(&point)
        .expect(format!("Failed to get cell at point {:?}", point).as_str());
      cell.set_collapsed(object_name);
    }
    debug!(
      "Generated path segment for chunk {} from {:?} to {:?} with [{}] cells: {}",
      cg,
      current_start,
      target_point,
      path_segment.len(),
      path_segment.iter().map(|p| format!("{:?}", p)).collect::<Vec<_>>().join(", ")
    );

    // Reset the grid and set the target as the new start for the next iteration
    object_grid.reset_path_grid();
    current_start = target_point;
  }

  // Find any collapsed cells that have more than two neighbours and update their object name which
  // is required as in the loop above we have no knowledge of any potential future path segments that may connect.
  let collapsed_cells = object_grid
    .object_grid
    .iter_mut()
    .flatten()
    .filter(|cell| cell.is_collapsed())
    .map(|cell| cell.ig.clone())
    .collect::<Vec<_>>();
  for point in collapsed_cells {
    let ig = object_grid.get_cell(&point).expect("Cell not found").ig;
    let neighbours = get_cardinal_direction_points(&ig)
      .into_iter()
      .filter_map(|(_, point)| {
        object_grid
          .get_cell(&point)
          .and_then(|cell| if cell.is_collapsed() { Some(cell) } else { None })
      })
      .collect::<Vec<_>>();
    if neighbours.len() > 2 {
      let neighbour_directions = neighbours
        .iter()
        .map(|n| Direction::from_points(&ig, &n.ig))
        .collect::<HashSet<_>>();
      let object_name = determine_path_object_name_from_all_neighbours(neighbour_directions);
      let cell = object_grid.get_cell_mut(&point).expect("Cell not found");
      cell.set_collapsed(object_name);
    }
  }

  debug!(
    "Generated complete path network for chunk {} with [{}] total cells across all segments",
    cg,
    path.len()
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

/// Determines the [`ObjectName`] for the path object based on the previous and next cell directions.
fn determine_path_object_name(
  mut previous_cell_direction: &Direction,
  mut next_cell_direction: &Direction,
  ig: &Point<InternalGrid>,
) -> ObjectName {
  next_cell_direction = update_if_edge_connection(ig, next_cell_direction);
  previous_cell_direction = update_if_edge_connection(ig, previous_cell_direction);
  match (previous_cell_direction, next_cell_direction) {
    (Direction::Top, Direction::Right) | (Direction::Right, Direction::Top) => ObjectName::PathTopRight,
    (Direction::Top, Direction::Bottom) | (Direction::Bottom, Direction::Top) => ObjectName::PathVertical,
    (Direction::Right, Direction::Left) | (Direction::Left, Direction::Right) => ObjectName::PathHorizontal,
    (Direction::Bottom, Direction::Left) | (Direction::Left, Direction::Bottom) => ObjectName::PathBottomLeft,
    (Direction::Bottom, Direction::Right) | (Direction::Right, Direction::Bottom) => ObjectName::PathBottomRight,
    (Direction::Top, Direction::Left) | (Direction::Left, Direction::Top) => ObjectName::PathTopLeft,
    (Direction::Top, Direction::Center) | (Direction::Center, Direction::Top) => ObjectName::PathTop,
    (Direction::Right, Direction::Center) | (Direction::Center, Direction::Right) => ObjectName::PathRight,
    (Direction::Bottom, Direction::Center) | (Direction::Center, Direction::Bottom) => ObjectName::PathBottom,
    (Direction::Left, Direction::Center) | (Direction::Center, Direction::Left) => ObjectName::PathLeft,
    _ => ObjectName::PathUndefined,
  }
}

/// Updates the direction if the point is at an edge of the internal grid. This is required because a point at the edge
/// signifies a connection point, meaning that the path does not end here but continues in the next chunk. Leaving the
/// direction as [`Direction::Center`] means that the path starts/ends here, which is never the case for a connection
/// point.
fn update_if_edge_connection<'a>(point: &Point<InternalGrid>, direction: &'a Direction) -> &'a Direction {
  if direction == Direction::Center {
    match point {
      point if point.x == 0 => &Direction::Left,
      point if point.x == CHUNK_SIZE - 1 => &Direction::Right,
      point if point.y == 0 => &Direction::Top,
      point if point.y == CHUNK_SIZE - 1 => &Direction::Bottom,
      _ => &Direction::Center,
    }
  } else {
    direction
  }
}

// TODO: Remove this or figure out a way to only update relevant directions i.e. the ones that currently don't connect
fn is_direction_relevant(ig: &Point<InternalGrid>, object_name: ObjectName, direction: &Direction) -> bool {
  match direction {
    _ => false,
  }
}

fn determine_path_object_name_from_all_neighbours(directions: HashSet<Direction>) -> ObjectName {
  use crate::generation::lib::Direction::*;
  let result = match directions.len() {
    3 => match &directions {
      d if d == &[Top, Right, Bottom].iter().cloned().collect() => ObjectName::PathRightVertical,
      d if d == &[Top, Left, Bottom].iter().cloned().collect() => ObjectName::PathLeftVertical,
      d if d == &[Left, Top, Right].iter().cloned().collect() => ObjectName::PathTopHorizontal,
      d if d == &[Left, Bottom, Right].iter().cloned().collect() => ObjectName::PathBottomHorizontal,
      _ => panic!("Unexpected combination of directions for path object name: {:?}", directions),
    },
    4 => ObjectName::PathCross,
    _ => ObjectName::PathUndefined,
  };
  debug!(
    "Resolved path object name from directions [{:?}] to [{:?}]",
    directions, result
  );

  result
}
