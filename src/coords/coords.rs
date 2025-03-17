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
    let world = Point::new_world_from_tile_grid(tg.clone());
    if w != world {
      panic!(
        "World coordinates do not match the tile grid coordinates - provided {} vs expected {} based on provided {}",
        w, world, tg
      );
    }
    Self {
      world,
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

#[cfg(test)]
mod tests {
  use super::*;
  use crate::constants::TILE_SIZE;
  use crate::coords::point::Point;

  #[test]
  fn new_creates_correct_coords() {
    let w = Point::new(10, 20);
    let cg = Point::new(1, 2);
    let tg = Point::new(3, 4);
    let coords = Coords::new(w, cg, tg);
    assert_eq!(coords.world, w);
    assert_eq!(coords.chunk_grid, cg);
    assert_eq!(coords.tile_grid, tg);
    assert_eq!(coords.internal_grid, Point::new(0, 0));
  }

  #[test]
  fn new_for_tile_creates_correct_coords() {
    let ig = Point::new_internal_grid(5, 6);
    let tg = Point::new_tile_grid(7, 8);
    let coords = Coords::new_for_tile(ig, tg);
    let w = Point::new_world(tg.x * TILE_SIZE as i32, tg.y * TILE_SIZE as i32);
    assert_eq!(coords.internal_grid, ig);
    assert_eq!(coords.tile_grid, tg);
    assert_eq!(coords.world, w);
    assert_eq!(coords.chunk_grid, Point::new_chunk_grid_from_world(w.clone()));
  }

  #[test]
  fn new_for_chunk_creates_correct_coords() {
    let w = Point::new_world(3 * TILE_SIZE as i32, 4 * TILE_SIZE as i32);
    let tg = Point::new_tile_grid(3, 4);
    let coords = Coords::new_for_chunk(w.clone(), tg.clone());
    assert_eq!(coords.world, w);
    assert_eq!(coords.chunk_grid, Point::new_chunk_grid_from_world(w.clone()));
    assert_eq!(coords.tile_grid, tg);
    assert_eq!(coords.internal_grid, Point::new_internal_grid(0, 0));
  }

  #[test]
  #[should_panic(expected = "World coordinates do not match the tile grid coordinates")]
  fn new_for_chunk_panics_on_mismatched_world_coords() {
    let w = Point::new(30, 40);
    let tg = Point::new(5, 6);
    Coords::new_for_chunk(w, tg);
  }

  #[test]
  fn debug_format_is_correct() {
    let coords = Coords::new(Point::new(10, 20), Point::new(1, 2), Point::new(3, 4));
    assert_eq!(format!("{:?}", coords), "[w(10, 20), cg(1, 2), tg(3, 4), ig(0, 0)]");
  }
}
