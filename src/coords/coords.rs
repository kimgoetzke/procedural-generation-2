use crate::constants::TILE_SIZE;
use crate::coords::point::*;
use std::fmt;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Coords {
  pub world: Point<World>,
  pub world_grid: Point<WorldGrid>,
  pub chunk_grid: Point<ChunkGrid>,
}

impl Coords {
  pub fn new(cg: Point<ChunkGrid>, wg: Point<WorldGrid>) -> Self {
    Self {
      world: Point::new_world_from_world_grid(wg.clone()),
      world_grid: wg,
      chunk_grid: cg,
    }
  }

  pub fn new_for_chunk(wg: Point<WorldGrid>) -> Self {
    Self {
      world: Point::new_world(wg.x * TILE_SIZE as i32, wg.y * TILE_SIZE as i32),
      world_grid: wg,
      chunk_grid: Point::new_chunk_grid(0, 0),
    }
  }
}

impl fmt::Debug for Coords {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "[w{}, wg{}, cg{}]", self.world, self.world_grid, self.chunk_grid)
  }
}
