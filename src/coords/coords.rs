use crate::constants::TILE_SIZE;
use crate::coords::point::*;
use std::fmt;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Default)]
pub struct Coords {
  pub world: Point<World>,
  pub chunk_grid: Point<ChunkGrid>,
  pub tile_grid: Point<TileGrid>,
  pub internal_grid: Point<InternalGrid>,
}

impl Coords {
  pub fn new(w: Point<World>, cg: Point<ChunkGrid>, tg: Point<TileGrid>) -> Self {
    Self {
      world: w,
      chunk_grid: cg,
      tile_grid: tg,
      internal_grid: Point::new(0, 0),
    }
  }

  pub fn new_for_tile(ig: Point<InternalGrid>, tg: Point<TileGrid>) -> Self {
    let w = Point::new_world_from_tile_grid(tg.clone());
    Self {
      chunk_grid: Point::new_chunk_grid_from_world(w.clone()),
      world: w,
      tile_grid: tg,
      internal_grid: ig,
    }
  }

  pub fn new_for_chunk(w: Point<World>, tg: Point<TileGrid>) -> Self {
    let cg = Point::new_chunk_grid_from_world(w.clone());
    let world = Point::new_world(tg.x * TILE_SIZE as i32, tg.y * TILE_SIZE as i32);
    if w != world {
      panic!("World coordinates do not match the tile grid coordinates");
    }
    Self {
      world: Point::new_world(tg.x * TILE_SIZE as i32, tg.y * TILE_SIZE as i32),
      chunk_grid: cg,
      tile_grid: tg,
      internal_grid: Point::new_internal_grid(0, 0),
    }
  }
}

impl fmt::Debug for Coords {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "[{}, {}, {}, {}]",
      self.world, self.chunk_grid, self.tile_grid, self.internal_grid
    )
  }
}
