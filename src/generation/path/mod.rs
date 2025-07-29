mod node;

use crate::constants::CHUNK_SIZE;
use crate::coords::Point;
use crate::coords::point::InternalGrid;
use crate::generation::lib::Chunk;
use crate::generation::path::node::{Node, NodeRef};
use crate::generation::resources::Metadata;
use bevy::app::{App, Plugin};
use bevy::log::*;
use bevy::prelude::Entity;
use std::rc::Rc;

pub struct PathGenerationPlugin;

impl Plugin for PathGenerationPlugin {
  fn build(&self, _: &mut App) {}
}

pub fn generate_paths(metadata: &Metadata, spawn_data: (Chunk, Entity)) -> Vec<Point<InternalGrid>> {
  let cg = spawn_data.0.coords.chunk_grid;
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
    return vec![];
  }
  if connection_points.len() != 2 {
    warn!(
      "Skipping path generation for chunk {} because MVP implementation is only for exactly 2 connection points but there are [{}]",
      cg,
      connection_points.len()
    );
    return vec![];
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
  // return vec![];

  // Create grid of nodes
  let mut node_grid = vec![vec![None; CHUNK_SIZE as usize]; CHUNK_SIZE as usize];
  let plane = spawn_data.0.layered_plane.flat;
  for x in 0..plane.data[0].len() {
    for y in 0..plane.data.len() {
      if let Some(tile) = &plane.data[x][y] {
        node_grid[x][y] = Some(Node::new(tile.coords.internal_grid));
      }
    }
  }

  // Populate neighbours for each node
  for x in 0..node_grid.len() {
    for y in 0..node_grid[x].len() {
      if let Some(node) = &node_grid[x][y] {
        let tile = plane.data[x][y].as_ref().expect("Tile should exist at this point");
        let mut neighbours = Vec::new();
        for p in vec![(-1, 1), (0, 1), (1, 1), (-1, 0), (1, 0), (-1, -1), (0, -1), (1, -1)].iter() {
          if let Some(neighbour) = plane.get_tile(Point::new_internal_grid(
            tile.coords.internal_grid.x + p.0,
            tile.coords.internal_grid.y + p.1,
          )) {
            let node_ref = node_grid[neighbour.coords.internal_grid.x as usize][neighbour.coords.internal_grid.y as usize]
              .as_ref()
              .expect("Neighbour node should exist at this point");
            neighbours.push(node_ref.clone());
          }
        }
        node.borrow_mut().add_neighbours(neighbours);
      }
    }
  }

  // Get start and target nodes based on connection points
  let start_node = node_grid[connection_points[0].x as usize][connection_points[0].y as usize]
    .as_ref()
    .expect("Start node should exist at this point")
    .clone();
  let target_node = node_grid[connection_points[1].x as usize][connection_points[1].y as usize]
    .as_ref()
    .expect("Start node should exist at this point")
    .clone();

  // Run the pathfinding algorithm and return the resulting path
  let path = run_algorithm(start_node, target_node);
  debug!(
    "Generated path for chunk {} with [{}] nodes in the path: {:?}",
    cg,
    path.len(),
    path
  );

  path
}

pub fn run_algorithm(start_node: NodeRef, target_node: NodeRef) -> Vec<Point<InternalGrid>> {
  let mut nodes_to_search: Vec<NodeRef> = vec![start_node.clone()];
  let mut processed: Vec<NodeRef> = Vec::new();

  while !nodes_to_search.is_empty() {
    // Find the node with the lowest F cost, using H cost as a tiebreaker
    let mut current: NodeRef = nodes_to_search[0].clone();
    for node in nodes_to_search.iter() {
      let node_f = node.borrow().get_f();
      let node_h = node.borrow().get_h();
      let current_f = current.borrow().get_f();
      let current_h = current.borrow().get_h();

      if node_f < current_f || (node_f == current_f && node_h < current_h) {
        current = node.clone();
      }
    }

    // Mark this node with the lowest F cost as processed
    processed.push(current.clone());
    nodes_to_search.retain(|n| !Rc::ptr_eq(n, &current));

    // If we have reached the target node, reconstruct the path and return it
    if Rc::ptr_eq(&current, &target_node) {
      debug!(
        "✅  Arrived at target node {:?}, now reconstructing the path",
        current.borrow().get_ig()
      );
      let mut path = Vec::new();
      let mut node = Some(current.clone());

      while let Some(current_node) = node {
        let (current_node_clone, next_ref) = {
          let cn = current_node.borrow();
          let next_ref = cn.get_connection().as_ref().cloned();
          let current_node_clone = cn.clone();

          (current_node_clone, next_ref)
        };
        path.push(current_node_clone.to_ig());
        node = next_ref;
      }

      path.reverse();
      return path;
    }

    // Otherwise, process the current node's neighbours
    let (current_g, current_ig, current_neighbours) = {
      let c = current.borrow();

      (c.get_g(), c.get_ig().clone(), c.get_neighbours().clone())
    };
    let target_ig: Point<InternalGrid> = get_target_node_ig(&target_node);
    debug!("Processing node at {:?}", current_ig);
    for neighbour in current_neighbours {
      let mut n = neighbour.borrow_mut();

      // Skip if the neighbour is already processed
      if processed.iter().any(|p| Rc::ptr_eq(p, &neighbour)) {
        trace!(
          " └─> Skipping neighbour {:?} because it has already been processed",
          n.get_ig()
        );
        continue;
      }

      // If the neighbour is not in the nodes to search or if the G cost to the neighbour is lower than its current
      // G cost...
      let is_not_in_nodes_to_search = !nodes_to_search.iter().any(|n| Rc::ptr_eq(n, &neighbour));
      let g_cost_to_neighbour = current_g + calculate_distance_cost(&current_ig, &n.get_ig());
      if is_not_in_nodes_to_search || g_cost_to_neighbour < n.get_g() {
        // ...then update the neighbour's G cost, set the current node as its connection
        n.set_g(g_cost_to_neighbour);
        n.set_connection(&current);
        let distance_cost = calculate_distance_cost(&n.get_ig(), &target_ig);

        // ...set the neighbour's H cost to the distance to the target node if it is not already in the nodes to search
        if is_not_in_nodes_to_search {
          n.set_h(distance_cost);
          nodes_to_search.push(neighbour.clone());
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

fn get_target_node_ig(target_node: &NodeRef) -> Point<InternalGrid> {
  target_node.borrow().get_ig().clone()
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
