use crate::constants::{CHUNK_SIZE, TILE_SIZE};
use crate::coords::Point;
use crate::coords::point::{ChunkGrid, CoordType, InternalGrid, TileGrid, World};
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

impl Direction {
  //noinspection DuplicatedCode
  pub fn from_points<T: CoordType + 'static>(a: &Point<T>, b: &Point<T>) -> Self {
    let (x_cmp, y_cmp) = (a.x.cmp(&b.x), a.y.cmp(&b.y));
    match std::any::TypeId::of::<T>() {
      id if id == std::any::TypeId::of::<InternalGrid>() => match (x_cmp, y_cmp) {
        (Ordering::Less, Ordering::Less) => Direction::BottomRight,
        (Ordering::Less, Ordering::Equal) => Direction::Right,
        (Ordering::Less, Ordering::Greater) => Direction::TopRight,
        (Ordering::Equal, Ordering::Less) => Direction::Bottom,
        (Ordering::Equal, Ordering::Equal) => Direction::Center,
        (Ordering::Equal, Ordering::Greater) => Direction::Top,
        (Ordering::Greater, Ordering::Less) => Direction::BottomLeft,
        (Ordering::Greater, Ordering::Equal) => Direction::Left,
        (Ordering::Greater, Ordering::Greater) => Direction::TopLeft,
      },
      _ => match (x_cmp, y_cmp) {
        (Ordering::Less, Ordering::Less) => Direction::TopRight,
        (Ordering::Less, Ordering::Equal) => Direction::Right,
        (Ordering::Less, Ordering::Greater) => Direction::BottomRight,
        (Ordering::Equal, Ordering::Less) => Direction::Top,
        (Ordering::Equal, Ordering::Equal) => Direction::Center,
        (Ordering::Equal, Ordering::Greater) => Direction::Bottom,
        (Ordering::Greater, Ordering::Less) => Direction::TopLeft,
        (Ordering::Greater, Ordering::Equal) => Direction::Left,
        (Ordering::Greater, Ordering::Greater) => Direction::BottomLeft,
      },
    }
  }

  pub fn from_chunk_w(chunk_world: &Point<World>, other_world: &Point<World>) -> Self {
    let chunk_len = CHUNK_SIZE * TILE_SIZE as i32;
    let chunk_left = chunk_world.x;
    let chunk_right = chunk_world.x + chunk_len - 1;
    let chunk_top = chunk_world.y;
    let chunk_bottom = chunk_world.y - chunk_len + 1;

    to_direction(other_world, chunk_left, chunk_right, chunk_top, chunk_bottom)
  }

  pub fn from_chunk_cg(chunk_world: &Point<ChunkGrid>, other_world: &Point<ChunkGrid>) -> Self {
    let chunk_left = chunk_world.x;
    let chunk_right = chunk_world.x - 1;
    let chunk_top = chunk_world.y;
    let chunk_bottom = chunk_world.y + 1;

    to_direction(other_world, chunk_left, chunk_right, chunk_top, chunk_bottom)
  }
}

impl PartialEq<Direction> for &Direction {
  fn eq(&self, other: &Direction) -> bool {
    **self == *other
  }
}

pub fn get_direction_points<T: CoordType + 'static>(point: &Point<T>) -> [(Direction, Point<T>); 9] {
  let (x_offset, y_offset) = calculate_offsets::<T>();
  let p = point;
  [
    (Direction::TopLeft, Point::new(p.x - x_offset, p.y + y_offset)),
    (Direction::Top, Point::new(p.x, p.y + y_offset)),
    (Direction::TopRight, Point::new(p.x + x_offset, p.y + y_offset)),
    (Direction::Left, Point::new(p.x - x_offset, p.y)),
    (Direction::Center, Point::new(p.x, p.y)),
    (Direction::Right, Point::new(p.x + x_offset, p.y)),
    (Direction::BottomLeft, Point::new(p.x - x_offset, p.y - y_offset)),
    (Direction::Bottom, Point::new(p.x, p.y - y_offset)),
    (Direction::BottomRight, Point::new(p.x + x_offset, p.y - y_offset)),
  ]
}

pub fn get_cardinal_direction_points<T: CoordType + 'static>(point: &Point<T>) -> [(Direction, Point<T>); 4] {
  let (x_offset, y_offset) = calculate_offsets::<T>();
  let p = point;
  [
    (Direction::Top, Point::new(p.x, p.y + y_offset)),
    (Direction::Left, Point::new(p.x - x_offset, p.y)),
    (Direction::Right, Point::new(p.x + x_offset, p.y)),
    (Direction::Bottom, Point::new(p.x, p.y - y_offset)),
  ]
}

/// Calculates the x and y offsets for a given coordinate type `T`. The returned tuple \(`(x_offset, y_offset)`\)
/// depends on the type parameter:
/// - For [`TileGrid`]: returns the chunk size for both x and y.
/// - For [`World`]: returns the product of tile size and chunk size for both x and y.
/// - For [`InternalGrid`]: returns (1, -1), reflecting its grid orientation.
/// - For [`ChunkGrid`]: returns (1, 1).
/// # Panics
/// If used with a coordinate type that is not implemented.
fn calculate_offsets<T: CoordType + 'static>() -> (i32, i32) {
  match std::any::TypeId::of::<T>() {
    id if id == std::any::TypeId::of::<TileGrid>() => (CHUNK_SIZE, CHUNK_SIZE),
    id if id == std::any::TypeId::of::<World>() => (TILE_SIZE as i32 * CHUNK_SIZE, TILE_SIZE as i32 * CHUNK_SIZE),
    id if id == std::any::TypeId::of::<InternalGrid>() => (1, -1),
    id if id == std::any::TypeId::of::<ChunkGrid>() => (1, 1),
    id => panic!("Coord type {:?} not implemented for calculate_offset", id),
  }
}

fn to_direction<T: CoordType>(other_world: &Point<T>, left: i32, right: i32, top: i32, bottom: i32) -> Direction {
  let x = if other_world.x < left {
    -1
  } else if other_world.x > right {
    1
  } else {
    0
  };
  let y = if other_world.y > top {
    1
  } else if other_world.y < bottom {
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

#[cfg(test)]
mod tests {
  use super::*;
  use crate::coords::Point;

  #[test]
  fn from_points_returns_correct_direction_for_internal_grid_1() {
    let a = Point::new_internal_grid(0, 0);
    let b = Point::new_internal_grid(1, 1);
    assert_eq!(Direction::from_points(&a, &b), Direction::BottomRight);

    let c = Point::new_internal_grid(0, 0);
    let d = Point::new_internal_grid(-1, -1);
    assert_eq!(Direction::from_points(&c, &d), Direction::TopLeft);

    let e = Point::new_internal_grid(0, 0);
    let f = Point::new_internal_grid(0, 0);
    assert_eq!(Direction::from_points(&e, &f), Direction::Center);
  }

  #[test]
  fn from_points_returns_correct_direction_for_internal_grid_2() {
    let a = Point::new_internal_grid(0, 0);
    for (direction, point) in get_direction_points(&a) {
      assert_eq!(Direction::from_points(&a, &point), direction);
    }
  }

  #[test]
  fn from_points_returns_correct_direction_for_tile_grid_1() {
    let a = Point::new_tile_grid(0, 0);
    let b = Point::new_tile_grid(1, 1);
    assert_eq!(Direction::from_points(&a, &b), Direction::TopRight);

    let c = Point::new_tile_grid(0, 0);
    let d = Point::new_tile_grid(-1, -1);
    assert_eq!(Direction::from_points(&c, &d), Direction::BottomLeft);

    let e = Point::new_tile_grid(0, 0);
    let f = Point::new_tile_grid(0, 0);
    assert_eq!(Direction::from_points(&e, &f), Direction::Center);
  }

  #[test]
  fn from_points_returns_correct_direction_for_tile_grid_2() {
    let a = Point::new_tile_grid(0, 0);
    for (direction, point) in get_direction_points(&a) {
      assert_eq!(Direction::from_points(&a, &point), direction);
    }
  }

  #[test]
  fn from_points_returns_correct_direction_for_chunk_grid_1() {
    let a = Point::new_chunk_grid(0, 0);
    let b = Point::new_chunk_grid(1, 1);
    assert_eq!(Direction::from_points(&a, &b), Direction::TopRight);

    let c = Point::new_chunk_grid(0, 0);
    let d = Point::new_chunk_grid(-1, -1);
    assert_eq!(Direction::from_points(&c, &d), Direction::BottomLeft);

    let e = Point::new_chunk_grid(0, 0);
    let f = Point::new_chunk_grid(0, 0);
    assert_eq!(Direction::from_points(&e, &f), Direction::Center);
  }

  #[test]
  fn from_points_returns_correct_direction_for_chunk_grid_2() {
    let a = Point::new_chunk_grid(0, 0);
    for (direction, point) in get_direction_points(&a) {
      assert_eq!(Direction::from_points(&a, &point), direction);
    }
  }

  #[test]
  fn from_points_returns_correct_direction_for_world_1() {
    let a = Point::new_world(0, 0);
    let b = Point::new_world(1, 1);
    assert_eq!(Direction::from_points(&a, &b), Direction::TopRight);

    let c = Point::new_world(0, 0);
    let d = Point::new_world(-1, -1);
    assert_eq!(Direction::from_points(&c, &d), Direction::BottomLeft);

    let e = Point::new_world(0, 0);
    let f = Point::new_world(0, 0);
    assert_eq!(Direction::from_points(&e, &f), Direction::Center);
  }

  #[test]
  fn from_points_returns_correct_direction_for_world_grid_2() {
    let a = Point::new_world(0, 0);
    for (direction, point) in get_direction_points(&a) {
      assert_eq!(Direction::from_points(&a, &point), direction);
    }
  }

  #[test]
  fn from_chunk_w_returns_correct_direction_for_adjacent_positive_chunk_w() {
    let distance = CHUNK_SIZE * TILE_SIZE as i32;
    let chunk_world = Point::new_world(0, 0);
    let other_world = Point::new_world(distance, distance);
    assert_eq!(Direction::from_chunk_w(&chunk_world, &other_world), Direction::TopRight);
  }

  #[test]
  fn from_chunk_w_returns_correct_direction_for_adjacent_negative_chunk_w() {
    let distance = CHUNK_SIZE * TILE_SIZE as i32;
    let chunk_world = Point::new_world(0, 0);
    let other_world = Point::new_world(-1 * distance, -1 * distance);
    assert_eq!(Direction::from_chunk_w(&chunk_world, &other_world), Direction::BottomLeft);
  }

  #[test]
  fn from_chunk_w_returns_correct_direction_for_distant_positive_chunk_w() {
    let distance = CHUNK_SIZE * TILE_SIZE as i32;
    let chunk_world = Point::new_world(0, 0);
    let other_world = Point::new_world(2 * distance, 4 * distance);
    assert_eq!(Direction::from_chunk_w(&chunk_world, &other_world), Direction::TopRight);
  }

  #[test]
  fn from_chunk_w_returns_correct_direction_for_distant_negative_chunk_w() {
    let distance = CHUNK_SIZE * TILE_SIZE as i32;
    let chunk_world = Point::new_world(0, 0);
    let other_world = Point::new_world(-3 * distance, -2 * distance);
    assert_eq!(Direction::from_chunk_w(&chunk_world, &other_world), Direction::BottomLeft);
  }

  #[test]
  fn from_chunk_cg_returns_correct_direction_for_adjacent_cg() {
    let first_cg = Point::new(0, 0);
    let second_cg = Point::new(1, 1);
    assert_eq!(Direction::from_chunk_cg(&first_cg, &second_cg), Direction::TopRight);

    let third_cg = Point::new(-1, -1);
    assert_eq!(Direction::from_chunk_cg(&first_cg, &third_cg), Direction::BottomLeft);
    assert_eq!(Direction::from_chunk_cg(&second_cg, &third_cg), Direction::BottomLeft);
  }

  #[test]
  fn from_chunk_cg_returns_correct_direction_for_distant_cg() {
    let first_cg = Point::new(0, 0);
    let second_cg = Point::new(10, 51);
    assert_eq!(Direction::from_chunk_cg(&first_cg, &second_cg), Direction::TopRight);

    let third_cg = Point::new(-25, -63);
    assert_eq!(Direction::from_chunk_cg(&first_cg, &third_cg), Direction::BottomLeft);
    assert_eq!(Direction::from_chunk_cg(&second_cg, &third_cg), Direction::BottomLeft);
  }

  #[test]
  fn get_direction_points_returns_correct_points_for_ig() {
    let point = Point::new_internal_grid(0, 0);
    let points = get_direction_points(&point);
    assert_eq!(points[0], (Direction::TopLeft, Point::new(-1, -1)));
    assert_eq!(points[4], (Direction::Center, Point::new(0, 0)));
    assert_eq!(points[8], (Direction::BottomRight, Point::new(1, 1)));
  }

  #[test]
  fn get_direction_points_returns_correct_points_for_tg() {
    let point = Point::new_tile_grid(0, 0);
    let points = get_direction_points(&point);
    assert_eq!(points[0], (Direction::TopLeft, Point::new(-16, 16)));
    assert_eq!(points[4], (Direction::Center, Point::new(0, 0)));
    assert_eq!(points[8], (Direction::BottomRight, Point::new(16, -16)));
  }

  #[test]
  fn get_direction_points_returns_correct_points_for_cg() {
    let point = Point::new_chunk_grid(0, 0);
    let points = get_direction_points(&point);
    assert_eq!(points[0], (Direction::TopLeft, Point::new(-1, 1)));
    assert_eq!(points[4], (Direction::Center, Point::new(0, 0)));
    assert_eq!(points[8], (Direction::BottomRight, Point::new(1, -1)));
  }

  #[test]
  fn get_direction_points_returns_correct_points_for_w() {
    let point = Point::new_world(0, 0);
    let points = get_direction_points(&point);
    assert_eq!(points[0], (Direction::TopLeft, Point::new(-512, 512)));
    assert_eq!(points[4], (Direction::Center, Point::new(0, 0)));
    assert_eq!(points[8], (Direction::BottomRight, Point::new(512, -512)));
  }

  #[test]
  fn get_cardinal_direction_points_returns_correct_points_for_ig() {
    let point = Point::new_internal_grid(0, 0);
    let points = get_cardinal_direction_points(&point);
    assert_eq!(points[0], (Direction::Top, Point::new(0, -1)));
    assert_eq!(points[1], (Direction::Left, Point::new(-1, 0)));
    assert_eq!(points[2], (Direction::Right, Point::new(1, 0)));
    assert_eq!(points[3], (Direction::Bottom, Point::new(0, 1)));
  }

  #[test]
  fn get_cardinal_direction_points_returns_correct_points_for_tg() {
    let point = Point::new_tile_grid(0, 0);
    let points = get_cardinal_direction_points(&point);
    assert_eq!(points[0], (Direction::Top, Point::new(0, 16)));
    assert_eq!(points[1], (Direction::Left, Point::new(-16, 0)));
    assert_eq!(points[2], (Direction::Right, Point::new(16, 0)));
    assert_eq!(points[3], (Direction::Bottom, Point::new(0, -16)));
  }

  #[test]
  fn get_cardinal_direction_points_returns_correct_points_for_cg() {
    let point = Point::new_chunk_grid(0, 0);
    let points = get_cardinal_direction_points(&point);
    assert_eq!(points[0], (Direction::Top, Point::new(0, 1)));
    assert_eq!(points[1], (Direction::Left, Point::new(-1, 0)));
    assert_eq!(points[2], (Direction::Right, Point::new(1, 0)));
    assert_eq!(points[3], (Direction::Bottom, Point::new(0, -1)));
  }

  #[test]
  fn get_cardinal_direction_points_returns_correct_points_for_w() {
    let point = Point::new_world(0, 0);
    let points = get_cardinal_direction_points(&point);
    assert_eq!(points[0], (Direction::Top, Point::new(0, 512)));
    assert_eq!(points[1], (Direction::Left, Point::new(-512, 0)));
    assert_eq!(points[2], (Direction::Right, Point::new(512, 0)));
    assert_eq!(points[3], (Direction::Bottom, Point::new(0, -512)));
  }
}
