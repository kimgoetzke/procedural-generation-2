use crate::constants::{CHUNK_SIZE, TILE_SIZE};
use crate::coords::{CoordType, Point};
use cmp::Ordering;
use std::cmp;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Direction {
  TopLeft,
  Top,
  TopRight,
  Left,
  Center,
  Right,
  BottomLeft,
  Bottom,
  BottomRight,
}

#[allow(dead_code)]
impl Direction {
  pub fn from_points(a: &Point, b: &Point) -> Self {
    match (a.x.cmp(&b.x), a.y.cmp(&b.y)) {
      (Ordering::Less, Ordering::Less) => Direction::TopRight,
      (Ordering::Less, Ordering::Equal) => Direction::Right,
      (Ordering::Less, Ordering::Greater) => Direction::BottomRight,
      (Ordering::Equal, Ordering::Less) => Direction::Top,
      (Ordering::Equal, Ordering::Equal) => Direction::Center,
      (Ordering::Equal, Ordering::Greater) => Direction::Bottom,
      (Ordering::Greater, Ordering::Less) => Direction::TopLeft,
      (Ordering::Greater, Ordering::Equal) => Direction::Left,
      (Ordering::Greater, Ordering::Greater) => Direction::BottomLeft,
    }
  }

  pub fn from_chunk(chunk_world: &Point, other_world: &Point) -> Self {
    if chunk_world.coord_type != CoordType::World || other_world.coord_type != CoordType::World {
      panic!("The provided coordinates must be of type 'World'");
    }

    let chunk_len = CHUNK_SIZE * TILE_SIZE as i32;
    let chunk_left = chunk_world.x;
    let chunk_right = chunk_world.x + chunk_len - 1;
    let chunk_top = chunk_world.y;
    let chunk_bottom = chunk_world.y - chunk_len + 1;
    let x = if other_world.x < chunk_left {
      -1
    } else if other_world.x > chunk_right {
      1
    } else {
      0
    };
    let y = if other_world.y > chunk_top {
      1
    } else if other_world.y < chunk_bottom {
      -1
    } else {
      0
    };

    match (x, y) {
      (-1, 1) => Direction::TopLeft,
      (0, 1) => Direction::Top,
      (1, 1) => Direction::TopRight,
      (-1, 0) => Direction::Left,
      (0, 0) => Direction::Center,
      (1, 0) => Direction::Right,
      (-1, -1) => Direction::BottomLeft,
      (0, -1) => Direction::Bottom,
      (1, -1) => Direction::BottomRight,
      _ => unreachable!("Reaching this was supposed to be impossible..."),
    }
  }
}

pub fn get_direction_points(point: &Point) -> [(Direction, Point); 9] {
  let ct = point.coord_type;
  let offset = match ct {
    CoordType::WorldGrid => CHUNK_SIZE,
    _ => panic!("Coord type {:?} not implemented for get_direction_points", ct),
  };
  let p = point;
  [
    (Direction::TopLeft, Point::new(p.x - offset, p.y + offset, ct)),
    (Direction::Top, Point::new(p.x, p.y + offset, ct)),
    (Direction::TopRight, Point::new(p.x + offset, p.y + offset, ct)),
    (Direction::Left, Point::new(p.x - offset, p.y, ct)),
    (Direction::Center, Point::new(p.x, p.y, ct)),
    (Direction::Right, Point::new(p.x + offset, p.y, ct)),
    (Direction::BottomLeft, Point::new(p.x - offset, p.y - offset, ct)),
    (Direction::Bottom, Point::new(p.x, p.y - offset, ct)),
    (Direction::BottomRight, Point::new(p.x + offset, p.y - offset, ct)),
  ]
}
