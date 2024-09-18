use crate::constants::{CHUNK_SIZE, TILE_SIZE};
use std::fmt;

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Coords {
  pub world: Point,
  pub grid: Point,
  pub chunk: Point,
}

impl Coords {
  pub fn new_grid_for_tile(chunk: Point, grid: Point) -> Self {
    Self {
      world: Point::new(grid.x * TILE_SIZE as i32, grid.y * TILE_SIZE as i32),
      grid,
      chunk,
    }
  }

  pub fn new_world_for_chunk(world: Point) -> Self {
    Self {
      grid: Point::new(world.x / CHUNK_SIZE, world.y / CHUNK_SIZE),
      world,
      chunk: Point::new(0, 0),
    }
  }
}

impl fmt::Debug for Coords {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "[w{}, g{}, c{}]", self.world, self.grid, self.chunk)
  }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Point {
  pub x: i32,
  pub y: i32,
}

impl fmt::Debug for Point {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "({}, {})", self.x, self.y)
  }
}

impl fmt::Display for Point {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "({}, {})", self.x, self.y)
  }
}

impl Point {
  pub fn new(x: i32, y: i32) -> Self {
    Self { x, y }
  }
}
