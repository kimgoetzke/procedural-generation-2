use crate::constants::TILE_SIZE;
use crate::generation::direction::Direction;
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
    if world_grid.coord_type != CoordType::WorldGrid {
      panic!("The provided coordinates must be of type 'WorldGrid'");
    }
    Self {
      world: Point {
        x: world_grid.x * TILE_SIZE as i32,
        y: world_grid.y * TILE_SIZE as i32,
        coord_type: CoordType::World,
      },
      world_grid,
      chunk_grid: Point::new_chunk_grid(0, 0),
    }
  }
}

impl fmt::Debug for Coords {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "[w{}, wg{}, cg{}]", self.world, self.world_grid, self.chunk_grid)
  }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Point {
  pub x: i32,
  pub y: i32,
  pub coord_type: CoordType,
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

impl Default for Point {
  fn default() -> Self {
    Self {
      x: 0,
      y: 0,
      coord_type: CoordType::World,
    }
  }
}

impl Point {
  pub fn new(x: i32, y: i32, coord_type: CoordType) -> Self {
    Self { x, y, coord_type }
  }

  pub fn new_abstract(x: i32, y: i32) -> Self {
    Self {
      x,
      y,
      coord_type: CoordType::Abstract,
    }
  }

  pub fn new_world(x: i32, y: i32) -> Self {
    Self {
      x,
      y,
      coord_type: CoordType::World,
    }
  }

  pub fn new_world_grid(x: i32, y: i32) -> Self {
    Self {
      x,
      y,
      coord_type: CoordType::WorldGrid,
    }
  }

  pub fn new_chunk_grid(x: i32, y: i32) -> Self {
    Self {
      x,
      y,
      coord_type: CoordType::ChunkGrid,
    }
  }

  /// Returns a `Point` of type `World` with the `x` and `y` values rounded to the nearest integer to achieve this.
  pub fn new_world_from_world_vec2(world: Vec2) -> Self {
    Self {
      x: world.x.round() as i32,
      y: world.y.round() as i32,
      coord_type: CoordType::World,
    }
  }

  pub fn new_world_from_world_grid(world_grid: Point) -> Self {
    Self {
      x: world_grid.x * TILE_SIZE as i32,
      y: world_grid.y * TILE_SIZE as i32,
      coord_type: CoordType::World,
    }
  }

  /// Returns a `Point` on the tile grid with the `x` and `y` values rounded to the nearest tile to achieve this. Used
  /// to convert world coordinates to grid coordinates.
  pub fn new_world_grid_from_world_vec2(world: Vec2) -> Self {
    Self {
      x: ((world.x - (TILE_SIZE as f32 / 2.)) / TILE_SIZE as f32).round() as i32,
      y: ((world.y + (TILE_SIZE as f32 / 2.)) / TILE_SIZE as f32).round() as i32,
      coord_type: CoordType::WorldGrid,
    }
  }

  pub fn new_world_grid_from_world(world: Point) -> Self {
    Self {
      x: (world.x as f32 / TILE_SIZE as f32).round() as i32,
      y: (world.y as f32 / TILE_SIZE as f32).round() as i32,
      coord_type: CoordType::WorldGrid,
    }
  }

  pub fn from_direction(direction: &Direction, coord_type: CoordType) -> Self {
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
    Self { x: i, y: j, coord_type }
  }

  pub fn distance_to(&self, other: &Point) -> f32 {
    assert_eq!(
      self.coord_type, other.coord_type,
      "Cannot calculate distance between points of different types"
    );
    ((self.x - other.x).pow(2) as f32 + (self.y - other.y).pow(2) as f32).sqrt()
  }

  pub fn to_vec2(&self) -> Vec2 {
    if self.coord_type == CoordType::World {
      return Vec2::new(self.x as f32, self.y as f32);
    }
    panic!("Cannot convert a Point of type {:?} to a Vec2", self.coord_type);
  }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum CoordType {
  World,
  WorldGrid,
  ChunkGrid,
  Abstract,
}
