use crate::constants::TILE_SIZE;
use crate::coords::point::*;
use std::fmt;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Default)]
pub struct Coords {
  pub world: Point<World>,
  pub tile_grid: Point<TileGrid>,
  pub internal_grid: Point<InternalGrid>,
}

impl Coords {
  pub fn new(ig: Point<InternalGrid>, tg: Point<TileGrid>) -> Self {
    Self {
      world: Point::new_world_from_tile_grid(tg.clone()),
      tile_grid: tg,
      internal_grid: ig,
    }
  }

  pub fn new_for_chunk(tg: Point<TileGrid>) -> Self {
    Self {
      world: Point::new_world(tg.x * TILE_SIZE as i32, tg.y * TILE_SIZE as i32),
      tile_grid: tg,
      internal_grid: Point::new_internal_grid(0, 0),
    }
  }
}

impl fmt::Debug for Coords {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "[{}, {}, {}]", self.world, self.tile_grid, self.internal_grid)
  }
}
