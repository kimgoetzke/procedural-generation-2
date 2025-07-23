mod node;

use crate::generation::path::node::{Node, NodeRef};
use bevy::app::{App, Plugin};
use std::rc::Rc;

pub struct PathGenerationPlugin;

impl Plugin for PathGenerationPlugin {
  fn build(&self, _: &mut App) {}
}

pub fn generate_path(start_node: NodeRef, target_node: NodeRef) -> Vec<Node> {
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
      let mut path = Vec::new();
      let mut node = Some(current.clone());

      while let Some(current_node) = node {
        let (current_node_clone, next_ref) = {
          let cn = current_node.borrow();
          let next_ref = cn.get_connection().as_ref().cloned();
          let current_node_clone = cn.clone();

          (current_node_clone, next_ref)
        };

        path.push(current_node_clone);
        node = next_ref;
      }
      path.reverse();
      return path;
    }

    // Otherwise, process the current node's neighbours
    for neighbour in current.borrow_mut().get_neighbours().iter() {
      let c = current.borrow();
      let mut n = neighbour.borrow_mut();
      let in_search = nodes_to_search.contains(&neighbour);
      let g_cost_to_neighbour = c.get_g() + c.distance_to(&n);

      if !in_search || g_cost_to_neighbour < n.get_g() {
        n.set_g(g_cost_to_neighbour);
        n.set_connection(&current);

        if !in_search {
          let distance = n.distance_to_ref(&target_node);
          n.set_h(distance);
          nodes_to_search.push(neighbour.clone());
        }
      }
    }
  }

  vec![]
}
