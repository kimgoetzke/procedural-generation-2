#![allow(dead_code)]

use crate::constants::{CHUNK_SIZE, TILE_SIZE};
use bevy::math::Vec2;
use std::fmt;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Coords {
  pub world: Point,
  pub world_grid: Point,
  pub chunk_grid: Point,
}

impl Coords {
  pub fn new(chunk_grid: Point, world_grid: Point) -> Self {
    Self {
      world: Point::new_world_from_world_grid(world_grid),
      world_grid,
      chunk_grid,
    }
  }

  pub fn new_for_chunk(world_grid: Point) -> Self {
    Self {
      world: Point::new_world_from_world_grid(world_grid),
      world_grid,
      chunk_grid: Point::new(0, 0),
    }
  }
}

impl fmt::Debug for Coords {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "[w{}, wg{}, cg{}]", self.world, self.world_grid, self.chunk_grid)
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

  /// Returns a `Point` in the world with the `x` and `y` values rounded to the nearest integer to achieve this.
  pub fn new_world_from_world_vec2(world: Vec2) -> Self {
    Self {
      x: world.x.round() as i32,
      y: world.y.round() as i32,
    }
  }

  pub fn new_world_from_world_grid(world_grid: Point) -> Self {
    Self {
      x: world_grid.x * TILE_SIZE as i32,
      y: world_grid.y * TILE_SIZE as i32,
    }
  }

  /// Returns a `Point` on the tile grid with the `x` and `y` values rounded to the nearest tile to achieve this. Used
  /// to convert world coordinates to grid coordinates.
  pub fn new_world_grid_from_world_vec2(world: Vec2) -> Self {
    Self {
      x: (world.x / TILE_SIZE as f32).round() as i32,
      y: (world.y / TILE_SIZE as f32).round() as i32,
    }
  }

  pub fn new_chunk_grid_from_world_vec2(world: Vec2) -> Self {
    Self {
      x: (world.x / CHUNK_SIZE as f32).round() as i32,
      y: (world.y / CHUNK_SIZE as f32).round() as i32,
    }
  }

  pub fn new_chunk_grid_from_world_point(world: Point) -> Self {
    Self {
      x: world.x / CHUNK_SIZE,
      y: world.y / CHUNK_SIZE,
    }
  }
}
