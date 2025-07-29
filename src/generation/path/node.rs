use crate::coords::Point;
use crate::coords::point::InternalGrid;
use std::cell::RefCell;
use std::rc::Rc;

pub(crate) type NodeRef = Rc<RefCell<Node>>;

#[derive(PartialEq, Debug, Clone)]
pub(crate) struct Node {
  ig: Point<InternalGrid>,
  neighbours: Vec<NodeRef>,
  connection: Box<Option<NodeRef>>,
  g: f32,
  h: f32,
}

impl Node {
  pub fn new(ig: Point<InternalGrid>) -> NodeRef {
    Rc::new(RefCell::new(Node {
      ig,
      neighbours: Vec::new(),
      connection: Box::new(None),
      g: 0.0,
      h: 0.0,
    }))
  }

  pub fn get_ig(&self) -> &Point<InternalGrid> {
    &self.ig
  }

  pub fn add_neighbour(&mut self, neighbour: NodeRef) {
    if !self.neighbours.contains(&neighbour) {
      self.neighbours.push(neighbour);
    }
  }

  pub fn add_neighbours(&mut self, neighbours: Vec<NodeRef>) {
    for neighbour in neighbours {
      self.add_neighbour(neighbour);
    }
  }

  pub fn get_neighbours(&self) -> &Vec<NodeRef> {
    &self.neighbours
  }

  /// Returns the [`NodeRef`] that this node is connected to, if any. Used to reconstruct the path from the start node
  /// to the target node after the pathfinding algorithm has completed.
  pub fn get_connection(&self) -> &Option<NodeRef> {
    &self.connection
  }

  /// Sets the connection to another [`NodeRef`], which is used to reconstruct the path from the start node to the
  /// target.
  pub fn set_connection(&mut self, connection: &NodeRef) {
    *self.connection = Some(connection.clone());
  }

  /// The distance from the start node to this node.
  pub fn get_g(&self) -> f32 {
    self.g
  }

  /// Sets the `G` cost which represents the distance from the start node to this node.
  pub fn set_g(&mut self, g: f32) {
    self.g = g;
  }

  /// The heuristic value, which is the estimated ("ideal") distance to reach the target node from this node. This
  /// value is always equal to or less than the actual distance to the target node.
  pub fn get_h(&self) -> f32 {
    self.h
  }

  /// Sets the `H` cost i.e. heuristic value, which is the estimated distance to reach the target node from this node.
  pub fn set_h(&mut self, h: f32) {
    self.h = h;
  }

  /// The total cost of this node, which is the sum of the distance from the start node (`G`) and the heuristic
  /// value (`H`).
  pub fn get_f(&self) -> f32 {
    self.g + self.h
  }

  /// Consumes the node and returns its internal grid coordinates for use by the wider crate.
  pub fn to_ig(self) -> Point<InternalGrid> {
    self.ig
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn new_sets_correct_ig() {
    let ig = Point::default();
    let node_ref = Node::new(ig.clone());
    assert_eq!(node_ref.borrow().get_ig(), &ig);
  }

  #[test]
  fn add_neighbour_only_adds_unique_neighbours() {
    let node_ref = Node::new(Point::default());
    let neighbour = Node::new(Point::default());
    node_ref.borrow_mut().add_neighbour(neighbour.clone());
    node_ref.borrow_mut().add_neighbour(neighbour.clone());
    assert_eq!(node_ref.borrow().get_neighbours().len(), 1);
  }

  #[test]
  fn add_neighbours_adds_multiple_unique_neighbours() {
    let node_ref = Node::new(Point::default());
    let neighbour1 = Node::new(Point::new_internal_grid(1, 1));
    let neighbour2 = Node::new(Point::new_internal_grid(2, 2));
    node_ref
      .borrow_mut()
      .add_neighbours(vec![neighbour1.clone(), neighbour2.clone(), neighbour1.clone()]);
    assert_eq!(node_ref.borrow().get_neighbours().len(), 2);
  }

  #[test]
  fn set_connection_updates_connection() {
    let node_ref = Node::new(Point::default());
    let connection1 = Node::new(Point::default());
    node_ref.borrow_mut().set_connection(&connection1);
    assert_eq!(node_ref.borrow().get_connection().as_ref(), Some(&connection1));

    let connection2 = Node::new(Point::new_internal_grid(4, 8));
    node_ref.borrow_mut().set_connection(&connection2);
    assert_eq!(node_ref.borrow().get_connection().as_ref(), Some(&connection2));
  }
}
