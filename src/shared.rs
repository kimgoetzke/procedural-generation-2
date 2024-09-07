use crate::settings::{CHUNK_SIZE, TILE_SIZE};
use bevy::utils::HashSet;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Chunk {
  pub coords: Coords,
  pub center: Point,
  pub tiles: HashSet<Tile>,
}

impl Chunk {
  pub fn new(world_location: Point, tiles: HashSet<Tile>) -> Self {
    Self {
      center: Point::new(world_location.x + (CHUNK_SIZE / 2), world_location.y + (CHUNK_SIZE / 2)),
      coords: Coords::new_world(world_location),
      tiles,
    }
  }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Tile {
  pub coords: Coords,
  pub tile_type: TileType,
  pub sprite_index: usize,
  pub layer: i32,
}

impl Tile {
  pub fn new(grid_location: Point, tile_type: TileType, sprite_index: usize, z_index: i32) -> Self {
    Self {
      coords: Coords::new_grid(grid_location, TILE_SIZE),
      tile_type,
      sprite_index,
      layer: z_index,
    }
  }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Coords {
  pub world: Point,
  pub grid: Point,
}

impl Coords {
  pub fn new_grid(grid: Point, tile_size: u32) -> Self {
    Self {
      world: Point::new(grid.x * tile_size as i32, grid.y * tile_size as i32),
      grid,
    }
  }

  pub fn new_world(world: Point) -> Self {
    Self {
      grid: Point::new(world.x, world.y),
      world,
    }
  }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Point {
  pub x: i32,
  pub y: i32,
}

impl Point {
  pub fn new(x: i32, y: i32) -> Self {
    Self { x, y }
  }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum TileType {
  Water,
  Sand,
  Grass,
  Forest,
}

pub fn get_chunk_neighbour_points(chunk: Chunk) -> [Point; 8] {
  [
    Point::new(
      (chunk.coords.world.x - 1) * (CHUNK_SIZE - 1),
      chunk.coords.world.y * chunk.coords.world.y * (CHUNK_SIZE - 1),
    ),
    Point::new(
      (chunk.coords.world.x + 1) * (CHUNK_SIZE - 1),
      chunk.coords.world.y * chunk.coords.world.y * (CHUNK_SIZE - 1),
    ),
    Point::new(
      chunk.coords.world.x * (CHUNK_SIZE - 1),
      (chunk.coords.world.x - 1) * (CHUNK_SIZE - 1),
    ),
    Point::new(
      chunk.coords.world.x * (CHUNK_SIZE - 1),
      (chunk.coords.world.y + 1) * (CHUNK_SIZE - 1),
    ),
    Point::new(
      (chunk.coords.world.x - 1) * (CHUNK_SIZE - 1),
      (chunk.coords.world.y + 1) * (CHUNK_SIZE - 1),
    ),
    Point::new(
      (chunk.coords.world.x + 1) * (CHUNK_SIZE - 1),
      (chunk.coords.world.y + 1) * (CHUNK_SIZE - 1),
    ),
    Point::new(
      (chunk.coords.world.x - 1) * (CHUNK_SIZE - 1),
      (chunk.coords.world.x - 1) * (CHUNK_SIZE - 1),
    ),
    Point::new(
      (chunk.coords.world.x + 1) * (CHUNK_SIZE - 1),
      (chunk.coords.world.y - 1) * (CHUNK_SIZE - 1),
    ),
  ]
}
