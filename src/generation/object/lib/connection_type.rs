use crate::constants::{CHUNK_SIZE, TILE_SIZE};
use crate::coords::point::{ChunkGrid, CoordType, World, WorldGrid};
use crate::coords::Point;

#[derive(serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash)]
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

pub fn get_connection_points<T: CoordType + 'static>(point: &Point<T>) -> [(Connection, Point<T>); 4] {
  let offset = match std::any::TypeId::of::<T>() {
    id if id == std::any::TypeId::of::<WorldGrid>() => CHUNK_SIZE,
    id if id == std::any::TypeId::of::<World>() => TILE_SIZE as i32 * CHUNK_SIZE,
    id if id == std::any::TypeId::of::<ChunkGrid>() => 1,
    _ => panic!("Coord type not implemented for get_connection_points"),
  };
  let p = point;
  [
    (Connection::Top, Point::new(p.x, p.y - offset)),
    (Connection::Left, Point::new(p.x - offset, p.y)),
    (Connection::Right, Point::new(p.x + offset, p.y)),
    (Connection::Bottom, Point::new(p.x, p.y + offset)),
  ]
}
