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

impl PartialEq<&Node> for Node {
  fn eq(&self, other: &&Node) -> bool {
    self.connection == other.connection && self.g == other.g && self.h == other.h && self.neighbours == other.neighbours
  }
}

impl Node {
  pub fn default() -> Self {
    Self {
      ig: Point::default(),
      neighbours: Vec::new(),
      connection: Box::new(None),
      g: 0.0,
      h: 0.0,
    }
  }

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
