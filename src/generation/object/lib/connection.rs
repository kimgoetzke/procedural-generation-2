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

pub fn get_connection_points(ig: &Point<InternalGrid>) -> [(Connection, Point<InternalGrid>); 4] {
  let point = ig;
  [
    (Connection::Top, Point::new(point.x, point.y - 1)),
    (Connection::Right, Point::new(point.x + 1, point.y)),
    (Connection::Bottom, Point::new(point.x, point.y + 1)),
    (Connection::Left, Point::new(point.x - 1, point.y)),
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

  #[test]
  fn get_connection_points_returns_correct_points_1() {
    let point = Point::new(5, 5);
    let connections = get_connection_points(&point);

    assert_eq!(connections[0], (Connection::Top, Point::new(5, 4)));
    assert_eq!(connections[1], (Connection::Right, Point::new(6, 5)));
    assert_eq!(connections[2], (Connection::Bottom, Point::new(5, 6)));
    assert_eq!(connections[3], (Connection::Left, Point::new(4, 5)));
  }

  #[test]
  fn get_connection_points_returns_correct_points_2() {
    let point = Point::new(0, 0);
    let connections = get_connection_points(&point);

    assert_eq!(connections[0], (Connection::Top, Point::new(0, -1)));
    assert_eq!(connections[1], (Connection::Right, Point::new(1, 0)));
    assert_eq!(connections[2], (Connection::Bottom, Point::new(0, 1)));
    assert_eq!(connections[3], (Connection::Left, Point::new(-1, 0)));
  }

  #[test]
  fn get_connection_points_handles_negative_coordinates_correctly() {
    let point = Point::new(-3, -3);
    let connections = get_connection_points(&point);

    assert_eq!(connections[0], (Connection::Top, Point::new(-3, -4)));
    assert_eq!(connections[1], (Connection::Right, Point::new(-2, -3)));
    assert_eq!(connections[2], (Connection::Bottom, Point::new(-3, -2)));
    assert_eq!(connections[3], (Connection::Left, Point::new(-4, -3)));
  }
}
