use crate::coords::Point;
use crate::coords::point::InternalGrid;
use bevy::reflect::Reflect;

#[derive(serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash, Reflect)]
pub enum Connection {
  Top,
  Right,
  Bottom,
  Left,
}

impl Connection {
  pub(crate) fn opposite(&self) -> Self {
    match self {
      Connection::Top => Connection::Bottom,
      Connection::Right => Connection::Left,
      Connection::Bottom => Connection::Top,
      Connection::Left => Connection::Right,
    }
  }
}

// TODO: Find out why left and right are inverted - is this making up for an unintended inversion elsewhere?
pub fn get_connection_points(point: &Point<InternalGrid>) -> [(Connection, Point<InternalGrid>); 4] {
  let p = point;
  [
    (Connection::Top, Point::new(p.x, p.y + 1)),
    (Connection::Right, Point::new(p.x - 1, p.y)),
    (Connection::Bottom, Point::new(p.x, p.y - 1)),
    (Connection::Left, Point::new(p.x + 1, p.y)),
  ]
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn opposite_of_top_is_bottom() {
    assert_eq!(Connection::Top.opposite(), Connection::Bottom);
  }

  #[test]
  fn opposite_of_bottom_is_top() {
    assert_eq!(Connection::Bottom.opposite(), Connection::Top);
  }

  #[test]
  fn opposite_of_right_is_left() {
    assert_eq!(Connection::Right.opposite(), Connection::Left);
  }

  #[test]
  fn opposite_of_left_is_right() {
    assert_eq!(Connection::Left.opposite(), Connection::Right);
  }
}
