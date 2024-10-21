use crate::constants::TILE_SIZE;
use crate::generation::lib::Direction;
use bevy::prelude::Vec2;
use bevy::reflect::{reflect_trait, Reflect};
use std::fmt;
use std::ops::Add;

#[reflect_trait]
pub trait CoordType {}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Reflect)]
pub struct World;

impl CoordType for World {}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Reflect)]
pub struct WorldGrid;

impl CoordType for WorldGrid {}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Reflect)]
pub struct ChunkGrid;

impl CoordType for ChunkGrid {}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Reflect)]
pub struct Point<T: CoordType> {
  pub x: i32,
  pub y: i32,
  #[reflect(ignore)]
  _marker: std::marker::PhantomData<T>,
}

impl<T: CoordType> fmt::Debug for Point<T> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "({}, {})", self.x, self.y)
  }
}

impl<T: CoordType> fmt::Display for Point<T> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "({}, {})", self.x, self.y)
  }
}

impl<T: CoordType> Default for Point<T> {
  fn default() -> Self {
    Self {
      x: 0,
      y: 0,
      _marker: std::marker::PhantomData,
    }
  }
}

impl<T: CoordType> Add for Point<T> {
  type Output = Self;

  fn add(self, other: Self) -> Self {
    Self {
      x: self.x + other.x,
      y: self.y + other.y,
      _marker: std::marker::PhantomData,
    }
  }
}

impl<T: CoordType> Point<T> {
  pub fn new(x: i32, y: i32) -> Self {
    Self {
      x,
      y,
      _marker: std::marker::PhantomData,
    }
  }

  pub const fn new_const(x: i32, y: i32) -> Self {
    Self {
      x,
      y,
      _marker: std::marker::PhantomData,
    }
  }

  pub fn from_direction(direction: &Direction) -> Self {
    let (i, j) = match direction {
      Direction::TopLeft => (-1, 1),
      Direction::Top => (0, 1),
      Direction::TopRight => (1, 1),
      Direction::Left => (-1, 0),
      Direction::Center => (0, 0),
      Direction::Right => (1, 0),
      Direction::BottomLeft => (-1, -1),
      Direction::Bottom => (0, -1),
      Direction::BottomRight => (1, -1),
    };
    Self::new(i, j)
  }

  pub fn distance_to(&self, other: &Point<T>) -> f32 {
    ((self.x - other.x).pow(2) as f32 + (self.y - other.y).pow(2) as f32).sqrt()
  }

  pub fn to_vec2(&self) -> Vec2 {
    Vec2::new(self.x as f32, self.y as f32)
  }
}

impl Point<World> {
  pub fn new_world(x: i32, y: i32) -> Self {
    Self::new(x, y)
  }

  /// Returns a `Point` of type `World` with the `x` and `y` values rounded to the nearest integer to achieve this.
  pub fn new_world_from_world_vec2(w: Vec2) -> Self {
    Self::new(w.x.round() as i32, w.y.round() as i32)
  }

  pub fn new_world_from_world_grid(wg: Point<WorldGrid>) -> Self {
    Self::new(wg.x * TILE_SIZE as i32, wg.y * TILE_SIZE as i32)
  }
}

impl Point<ChunkGrid> {
  pub fn new_chunk_grid(x: i32, y: i32) -> Self {
    Self::new(x, y)
  }
}

impl Point<WorldGrid> {
  pub fn new_world_grid(x: i32, y: i32) -> Self {
    Self::new(x, y)
  }

  /// Returns a `Point` on the tile grid with the `x` and `y` values rounded to the nearest tile to achieve this. Used
  /// to convert world coordinates to grid coordinates.
  pub fn new_world_grid_from_world_vec2(w: Vec2) -> Self {
    Self::new(
      ((w.x - (TILE_SIZE as f32 / 2.)) / TILE_SIZE as f32).round() as i32,
      ((w.y + (TILE_SIZE as f32 / 2.)) / TILE_SIZE as f32).round() as i32,
    )
  }

  pub fn new_world_grid_from_world(w: Point<World>) -> Self {
    Self::new(
      (w.x as f32 / TILE_SIZE as f32).round() as i32,
      (w.y as f32 / TILE_SIZE as f32).round() as i32,
    )
  }
}
