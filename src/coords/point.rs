use crate::constants::{CHUNK_SIZE, TILE_SIZE};
use crate::generation::lib::Direction;
use bevy::prelude::Vec2;
use bevy::reflect::{reflect_trait, Reflect};
use std::fmt;
use std::ops::Add;

#[reflect_trait]
pub trait CoordType {
  fn type_name() -> &'static str
  where
    Self: Sized;
}

/// Represents the world coordinates of the application. Like every `Point`, it stores the `x` and `y` values as `i32`.
/// Each `x`-`y` value pair represents a pixel in the world.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Reflect)]
pub struct World;

impl CoordType for World {
  fn type_name() -> &'static str {
    "w"
  }
}

/// Represents coordinates in the tile grid abstraction over the world coordinates. Each `Point` of type `TileGrid`
/// represents a tile of `TILE_SIZE` in the world.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Reflect)]
pub struct TileGrid;

impl CoordType for TileGrid {
  fn type_name() -> &'static str {
    "tg"
  }
}

/// Represents coordinates in the tile grid abstraction over the world coordinates. Each `Point` of type `ChunkGrid`
/// represents a chunk of `TILE_SIZE` * `CHUNK_SIZE` in the world.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Reflect)]
pub struct ChunkGrid;

impl CoordType for ChunkGrid {
  fn type_name() -> &'static str {
    "cg"
  }
}

/// Represents coordinates internal to any type of grid structure that uses them. `Point<InternalGrid>` differ from
/// other `Point`s in that the top left corner of the structure in which they are used is (0, 0) and the `x` and `y`
/// values increase towards the bottom right corner, whereas all other `Point`s are based on the world coordinates i.e.
/// not linked to structure that uses them.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Reflect)]
pub struct InternalGrid;

impl CoordType for InternalGrid {
  fn type_name() -> &'static str {
    "ig"
  }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Reflect)]
pub struct Point<T: CoordType> {
  pub x: i32,
  pub y: i32,
  #[reflect(ignore)]
  _marker: std::marker::PhantomData<T>,
}

impl<T: CoordType> fmt::Debug for Point<T> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}({}, {})", T::type_name(), self.x, self.y)
  }
}

impl<T: CoordType> fmt::Display for Point<T> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}({}, {})", T::type_name(), self.x, self.y)
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

  pub fn new_world_from_chunk_grid(cg: Point<ChunkGrid>) -> Self {
    Self::new(cg.x * CHUNK_SIZE * TILE_SIZE as i32, cg.y * CHUNK_SIZE * TILE_SIZE as i32)
  }

  pub fn new_world_from_tile_grid(tg: Point<TileGrid>) -> Self {
    Self::new(tg.x * TILE_SIZE as i32, tg.y * TILE_SIZE as i32)
  }
}

impl Point<InternalGrid> {
  /// Creates new `Point` of type `InternalGrid` whereby the top left corner of the grid is (0, 0) and x and y values
  /// increase towards the bottom right corner.
  pub fn new_internal_grid(x: i32, y: i32) -> Self {
    Self::new(x, y)
  }
}

impl Point<TileGrid> {
  pub fn new_tile_grid(x: i32, y: i32) -> Self {
    Self::new(x, y)
  }

  /// Returns a `Point` on the tile grid with the `x` and `y` values rounded to the nearest tile to achieve this. Used
  /// to convert world coordinates to tile grid coordinates.
  pub fn new_tile_grid_from_world_vec2(w: Vec2) -> Self {
    Self::new(
      ((w.x - (TILE_SIZE as f32 / 2.)) / TILE_SIZE as f32).round() as i32,
      ((w.y + (TILE_SIZE as f32 / 2.)) / TILE_SIZE as f32).round() as i32,
    )
  }

  pub fn new_tile_grid_from_world(w: Point<World>) -> Self {
    Self::new(
      (w.x as f32 / TILE_SIZE as f32).round() as i32,
      (w.y as f32 / TILE_SIZE as f32).round() as i32,
    )
  }
}

impl Point<ChunkGrid> {
  pub fn new_chunk_grid(x: i32, y: i32) -> Self {
    Self::new(x, y)
  }

  /// Returns a `Point` on the chunk grid with the `x` and `y` values rounded to the nearest chunk to achieve this. Used
  /// to convert world coordinates to chunk grid coordinates.
  pub fn new_chunk_grid_from_world_vec2(w: Vec2) -> Self {
    Self::new(
      (w.x / (TILE_SIZE as f32 * CHUNK_SIZE as f32)).round() as i32,
      (w.y / (TILE_SIZE as f32 * CHUNK_SIZE as f32)).round() as i32,
    )
  }

  pub fn new_chunk_grid_from_world(w: Point<World>) -> Self {
    Self::new(
      ((w.x as f32 + 1.) / (TILE_SIZE as f32 * CHUNK_SIZE as f32)).round() as i32,
      ((w.y as f32 - 1.) / (TILE_SIZE as f32 * CHUNK_SIZE as f32)).round() as i32,
    )
  }
}
