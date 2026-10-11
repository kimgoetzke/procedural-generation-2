use crate::constants::{CHUNK_SIZE, TILE_SIZE};
use crate::coordinates::Point;
use crate::coordinates::point::{ChunkGrid, CoordType, InternalGrid, TileGrid, World};
use cmp::Ordering;
use std::any::TypeId;
use std::cmp;

#[derive(serde::Deserialize, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Direction {
  TopLeft,
  Top,
  TopRight,
  Left,
  Centre,
  Right,
  BottomLeft,
  Bottom,
  BottomRight,
}

impl Direction {
  //noinspection DuplicatedCode
  pub fn from_points<T: CoordType + 'static>(a: &Point<T>, b: &Point<T>) -> Self {
    let (x_cmp, y_cmp) = (a.x.cmp(&b.x), a.y.cmp(&b.y));
    match TypeId::of::<T>() {
      id if id == TypeId::of::<InternalGrid>() => match (x_cmp, y_cmp) {
        (Ordering::Less, Ordering::Less) => Self::BottomRight,
        (Ordering::Less, Ordering::Equal) => Self::Right,
        (Ordering::Less, Ordering::Greater) => Self::TopRight,
        (Ordering::Equal, Ordering::Less) => Self::Bottom,
        (Ordering::Equal, Ordering::Equal) => Self::Centre,
        (Ordering::Equal, Ordering::Greater) => Self::Top,
        (Ordering::Greater, Ordering::Less) => Self::BottomLeft,
        (Ordering::Greater, Ordering::Equal) => Self::Left,
        (Ordering::Greater, Ordering::Greater) => Self::TopLeft,
      },
      _ => match (x_cmp, y_cmp) {
        (Ordering::Less, Ordering::Less) => Self::TopRight,
        (Ordering::Less, Ordering::Equal) => Self::Right,
        (Ordering::Less, Ordering::Greater) => Self::BottomRight,
        (Ordering::Equal, Ordering::Less) => Self::Top,
        (Ordering::Equal, Ordering::Equal) => Self::Centre,
        (Ordering::Equal, Ordering::Greater) => Self::Bottom,
        (Ordering::Greater, Ordering::Less) => Self::TopLeft,
        (Ordering::Greater, Ordering::Equal) => Self::Left,
        (Ordering::Greater, Ordering::Greater) => Self::BottomLeft,
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

  pub const fn to_opposite(self) -> Self {
    match self {
      Self::TopLeft => Self::BottomRight,
      Self::Top => Self::Bottom,
      Self::TopRight => Self::BottomLeft,
      Self::Left => Self::Right,
      Self::Centre => Self::Centre,
      Self::Right => Self::Left,
      Self::BottomLeft => Self::TopRight,
      Self::Bottom => Self::Top,
      Self::BottomRight => Self::TopLeft,
    }
  }

  pub fn to_point<T: CoordType + 'static>(self) -> Point<T> {
    match (TypeId::of::<T>(), self) {
      (id, Self::TopLeft) if id == TypeId::of::<InternalGrid>() => Point::new(-1, -1),
      (_, Self::TopLeft) => Point::new(-1, 1),
      (id, Self::Top) if id == TypeId::of::<InternalGrid>() => Point::new(0, -1),
      (_, Self::Top) => Point::new(0, 1),
      (id, Self::TopRight) if id == TypeId::of::<InternalGrid>() => Point::new(1, -1),
      (_, Self::TopRight) => Point::new(1, 1),
      (_, Self::Left) => Point::new(-1, 0),
      (_, Self::Centre) => Point::new(0, 0),
      (_, Self::Right) => Point::new(1, 0),
      (id, Self::BottomLeft) if id == TypeId::of::<InternalGrid>() => Point::new(-1, 1),
      (_, Self::BottomLeft) => Point::new(-1, -1),
      (id, Self::Bottom) if id == TypeId::of::<InternalGrid>() => Point::new(0, 1),
      (_, Self::Bottom) => Point::new(0, -1),
      (id, Self::BottomRight) if id == TypeId::of::<InternalGrid>() => Point::new(1, 1),
      (_, Self::BottomRight) => Point::new(1, -1),
    }
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
    (Direction::Centre, Point::new(p.x, p.y)),
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
  match TypeId::of::<T>() {
    id if id == TypeId::of::<TileGrid>() => (CHUNK_SIZE, CHUNK_SIZE),
    id if id == TypeId::of::<World>() => (TILE_SIZE as i32 * CHUNK_SIZE, TILE_SIZE as i32 * CHUNK_SIZE),
    id if id == TypeId::of::<InternalGrid>() => (1, -1),
    id if id == TypeId::of::<ChunkGrid>() => (1, 1),
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
    (0, 0) => Direction::Centre,
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
  use crate::coordinates::Point;

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
    assert_eq!(Direction::from_points(&e, &f), Direction::Centre);
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
    assert_eq!(Direction::from_points(&e, &f), Direction::Centre);
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
    assert_eq!(Direction::from_points(&e, &f), Direction::Centre);
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
    assert_eq!(Direction::from_points(&e, &f), Direction::Centre);
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
    let other_world = Point::new_world(-distance, -distance);
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
    assert_eq!(points[4], (Direction::Centre, Point::new(0, 0)));
    assert_eq!(points[8], (Direction::BottomRight, Point::new(1, 1)));
  }

  #[test]
  fn get_direction_points_returns_correct_points_for_tg() {
    let point = Point::new_tile_grid(0, 0);
    let points = get_direction_points(&point);
    assert_eq!(points[0], (Direction::TopLeft, Point::new(-16, 16)));
    assert_eq!(points[4], (Direction::Centre, Point::new(0, 0)));
    assert_eq!(points[8], (Direction::BottomRight, Point::new(16, -16)));
  }

  #[test]
  fn get_direction_points_returns_correct_points_for_cg() {
    let point = Point::new_chunk_grid(0, 0);
    let points = get_direction_points(&point);
    assert_eq!(points[0], (Direction::TopLeft, Point::new(-1, 1)));
    assert_eq!(points[4], (Direction::Centre, Point::new(0, 0)));
    assert_eq!(points[8], (Direction::BottomRight, Point::new(1, -1)));
  }

  #[test]
  fn get_direction_points_returns_correct_points_for_w() {
    let point = Point::new_world(0, 0);
    let points = get_direction_points(&point);
    assert_eq!(points[0], (Direction::TopLeft, Point::new(-512, 512)));
    assert_eq!(points[4], (Direction::Centre, Point::new(0, 0)));
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

  #[test]
  fn to_point_returns_correct_point_for_top_left_internal_grid() {
    let direction = Direction::TopLeft;
    let point: Point<InternalGrid> = direction.to_point();
    assert_eq!(point, Point::new(-1, -1));
  }

  #[test]
  fn to_point_returns_correct_point_for_top_left_other_grids() {
    let direction = Direction::TopLeft;
    let point: Point<TileGrid> = direction.to_point();
    assert_eq!(point, Point::new(-1, 1));
  }

  #[test]
  fn to_point_returns_correct_point_for_centre() {
    let direction = Direction::Centre;
    let point: Point<InternalGrid> = direction.to_point();
    assert_eq!(point, Point::new(0, 0));
  }

  #[test]
  fn to_point_returns_correct_point_for_bottom_right_internal_grid() {
    let direction = Direction::BottomRight;
    let point: Point<InternalGrid> = direction.to_point();
    assert_eq!(point, Point::new(1, 1));
  }

  #[test]
  fn to_point_returns_correct_point_for_bottom_right_other_grids() {
    let direction = Direction::BottomRight;
    let point: Point<World> = direction.to_point();
    assert_eq!(point, Point::new(1, -1));
  }
}
