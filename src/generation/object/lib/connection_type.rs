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

// TODO: Find out why left/right appear to be inverted - is there a bug elsewhere?
pub fn get_connection_points(point: &Point<InternalGrid>) -> [(Connection, Point<InternalGrid>); 4] {
  let p = point;
  [
    (Connection::Top, Point::new(p.x, p.y + 1)),
    (Connection::Left, Point::new(p.x + 1, p.y)),
    (Connection::Right, Point::new(p.x - 1, p.y)),
    (Connection::Bottom, Point::new(p.x, p.y - 1)),
  ]
}
