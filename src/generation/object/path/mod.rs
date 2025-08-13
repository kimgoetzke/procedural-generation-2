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
use std::collections::HashMap;
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
    .expect(format!("Failed to get connection points for {}", cg).as_str())
    .iter()
    .filter(|p| {
      if let Some(cell) = object_grid.get_cell_mut(&p) {
        cell.calculate_is_walkable();
        if cell.is_valid_connection_point() {
          // Remove when done:
          if let Some(tile_below) = &cell.tile_below {
            debug!("Keeping chunk {} connection point {:?} as a valid connection", cg, p,);
            tile_below.log();
          }

          return true;
        }
        debug!(
          "Removing chunk {} connection point {:?} because is walkable={} & is_valid_connection_point={}",
          cg,
          p,
          cell.is_walkable(),
          cell.is_valid_connection_point()
        );
        if let Some(tile_below) = &cell.tile_below {
          tile_below.log();
        } else {
          debug!("- No tile below for connection point {:?}", p);
        }

        return false;
      }
      debug!(
        "Removing chunk {} connection point {:?} because there is no tile in the object grid",
        cg, p
      );

      false
    })
    .collect::<Vec<_>>();

  if connection_points.is_empty() {
    debug!("Skipped path generation for chunk {} because it has no connection points", cg);
    return object_grid;
  }
  if connection_points.len() == 1 {
    if !is_edge_connection_point(connection_points[0]) {
      debug!(
        "Skipped path generation for chunk {} because it has no edge connection points",
        cg
      );
      return object_grid;
    }
    let cell = object_grid
      .get_cell_mut(&connection_points[0])
      .expect("Failed to get cell for connection point");
    let direction = direction_to_neighbour_chunk(connection_points[0]);
    let object_name = determine_path_object_name_from_neighbours(HashSet::from([direction]), connection_points[0]);
    cell.set_collapsed(object_name);
    debug!(
      "Skipped path generation for chunk {} because it has only 1 connection point",
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
      debug!(
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
  let counts = path.iter().fold(HashMap::new(), |mut acc, (point, _)| {
    *acc.entry(point).or_insert(0) += 1;
    acc
  });
  let cells_requiring_update: HashSet<(Point<InternalGrid>, Vec<Point<InternalGrid>>)> = path
    .iter()
    .map(|(point, _)| {
      let cell = object_grid.get_cell(point).expect("Cell not found");
      let neighbours = get_cardinal_direction_points(&cell.ig)
        .into_iter()
        .filter_map(|(_, point)| object_grid.get_cell(&point))
        .filter(|cell| cell.is_collapsed())
        .map(|cell| cell.ig.clone())
        .collect::<Vec<_>>();
      (point.clone(), neighbours)
    })
    .filter(|(point, _)| counts.get(point).copied().unwrap_or(0) > 1)
    .filter(|(_, neighbours)| neighbours.len() > 1)
    .collect::<HashSet<_>>();
  if !cells_requiring_update.is_empty() {
    debug!(
      "Found [{}] path cells where the object name needs to be updated: {:?}",
      cells_requiring_update.len(),
      cells_requiring_update
        .iter()
        .map(|(point, neighbours)| format!("{:?} -> {:?}", point, neighbours))
        .collect::<Vec<_>>()
        .join(", ")
    );
    for (point, neighbours) in cells_requiring_update {
      let neighbour_directions = neighbours
        .iter()
        .map(|n| {
          let cell = object_grid.get_cell(n).expect("Cell not found");
          Direction::from_points(&point, &cell.ig)
        })
        .collect::<HashSet<_>>();
      let cell = object_grid.get_cell_mut(&point).expect("Cell not found");
      let object_name = determine_path_object_name_from_neighbours(neighbour_directions, &cell.ig);
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
    let (current_g, current_ig, current_walkable_neighbours) = {
      let c = current_cell.lock().expect("Failed to lock current cell");

      (c.get_g(), c.ig, c.get_walkable_neighbours().clone())
    };
    let target_ig = get_cell_ig(target_cell);

    trace!("Processing cell at {:?}", current_ig);
    for neighbour in current_walkable_neighbours {
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

  let mut result = Vec::new();
  push_path_if_valid(start_cell, &mut result);
  push_path_if_valid(target_cell, &mut result);
  result
}

fn push_path_if_valid(cell: &CellRef, result: &mut Vec<(Point<InternalGrid>, Direction)>) {
  let ig = get_cell_ig(cell);
  if is_edge_connection_point(&ig) {
    result.push((ig, Direction::Center));
  }
}

fn is_edge_connection_point(ig: &Point<InternalGrid>) -> bool {
  ig.x == 0 || ig.x == CHUNK_SIZE - 1 || ig.y == 0 || ig.y == CHUNK_SIZE - 1
}

fn get_cell_ig(cell: &CellRef) -> Point<InternalGrid> {
  cell.lock().expect("Failed to lock cell").get_ig().clone()
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
  use crate::generation::lib::Direction::*;
  next_cell_direction = update_if_edge_connection(ig, next_cell_direction);
  previous_cell_direction = update_if_edge_connection(ig, previous_cell_direction);
  match (previous_cell_direction, next_cell_direction) {
    (Top, Right) | (Right, Top) => ObjectName::PathTopRight,
    (Top, Bottom) | (Bottom, Top) => ObjectName::PathVertical,
    (Right, Left) | (Left, Right) => ObjectName::PathHorizontal,
    (Bottom, Left) | (Left, Bottom) => ObjectName::PathBottomLeft,
    (Bottom, Right) | (Right, Bottom) => ObjectName::PathBottomRight,
    (Top, Left) | (Left, Top) => ObjectName::PathTopLeft,
    (Top, Center) | (Center, Top) | (Top, Top) => ObjectName::PathTop,
    (Right, Center) | (Center, Right) | (Right, Right) => ObjectName::PathRight,
    (Bottom, Center) | (Center, Bottom) | (Bottom, Bottom) => ObjectName::PathBottom,
    (Left, Center) | (Center, Left) | (Left, Left) => ObjectName::PathLeft,
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

fn determine_path_object_name_from_neighbours(
  mut neighbour_directions: HashSet<Direction>,
  ig: &Point<InternalGrid>,
) -> ObjectName {
  use crate::generation::lib::Direction::*;
  // If we only have two directions, then this an edge cell, so we need to add the direction to the connection point
  // in the neighbouring chunk
  if neighbour_directions.len() == 2 {
    neighbour_directions.insert(direction_to_neighbour_chunk(&ig));
  }
  let result = match neighbour_directions.len() {
    1 => match neighbour_directions.iter().next() {
      Some(Top) => ObjectName::PathTop,
      Some(Right) => ObjectName::PathRight,
      Some(Bottom) => ObjectName::PathBottom,
      Some(Left) => ObjectName::PathLeft,
      _ => unreachable!("Unexpected single direction for path object name: {:?}", neighbour_directions),
    },
    3 => match &neighbour_directions {
      d if d == &[Top, Right, Bottom].iter().cloned().collect() => ObjectName::PathRightVertical,
      d if d == &[Top, Left, Bottom].iter().cloned().collect() => ObjectName::PathLeftVertical,
      d if d == &[Left, Top, Right].iter().cloned().collect() => ObjectName::PathTopHorizontal,
      d if d == &[Left, Bottom, Right].iter().cloned().collect() => ObjectName::PathBottomHorizontal,
      _ => unreachable!(
        "Unexpected combination of directions for path object name: {:?}",
        neighbour_directions
      ),
    },
    4 => ObjectName::PathCross,
    _ => ObjectName::PathUndefined,
  };
  trace!(
    "Resolved path object name from directions [{:?}] to [{:?}]",
    neighbour_directions, result
  );

  result
}

fn direction_to_neighbour_chunk(ig: &Point<InternalGrid>) -> Direction {
  match ig {
    point if point.x == 0 => Direction::Left,
    point if point.x == CHUNK_SIZE - 1 => Direction::Right,
    point if point.y == 0 => Direction::Top,
    point if point.y == CHUNK_SIZE - 1 => Direction::Bottom,
    _ => Direction::Center,
  }
}
