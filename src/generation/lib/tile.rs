use crate::constants::{BUFFER_SIZE, CHUNK_SIZE, TILE_SIZE};
use crate::coords::point::{InternalGrid, World};
use crate::coords::{Coords, Point};
use crate::generation::lib::{DraftTile, TerrainType, TileType};
use bevy::log::*;
use bevy::reflect::Reflect;

/// A `Tile` represents a single tile of `TILE_SIZE` in the world. It contains information about its `Coords`,
/// `TerrainType`, `TileType`, and layer. If created from a `DraftTile`, the `layer` of a `Tile` adds the y-coordinate
/// of the world grid `Coords` to the layer from the `DraftTile` from which it was created. It also adjusts the
/// `InternalGrid` `Coords` to account for the buffer of a `DraftChunk` i.e. it shifts the `InternalGrid` `Coords` by the
/// `BUFFER_SIZE` to towards the top-left, allowing for the outer tiles of a `DraftChunk` to be cut off in a way that
/// the `Tile`s in the resulting `Chunk` have `InternalGrid` `Coords` ranging from 0 to `CHUNK_SIZE`.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Reflect)]
pub struct Tile {
  #[reflect(ignore)]
  pub coords: Coords,
  pub terrain: TerrainType,
  pub layer: i32,
  pub tile_type: TileType,
}

impl Tile {
  pub fn from(draft_tile: DraftTile, tile_type: TileType) -> Self {
    let adjusted_ig = Point::new_internal_grid(
      draft_tile.coords.internal_grid.x - BUFFER_SIZE,
      draft_tile.coords.internal_grid.y - BUFFER_SIZE,
    );
    let adjusted_coords = Coords::new(adjusted_ig, draft_tile.coords.tile_grid);
    if !is_marked_for_deletion(&adjusted_ig) {
      trace!(
        "Converting: DraftTile {:?} => {:?} {:?} tile {:?}",
        draft_tile.coords,
        tile_type,
        draft_tile.terrain,
        adjusted_coords,
      );
    }
    Self {
      coords: adjusted_coords,
      terrain: draft_tile.terrain,
      layer: draft_tile.layer + draft_tile.coords.internal_grid.y,
      tile_type,
    }
  }

  pub fn get_parent_chunk_w(&self) -> Point<World> {
    Point::new_world(
      (self.coords.tile_grid.x - self.coords.internal_grid.x) * TILE_SIZE as i32,
      (self.coords.tile_grid.y + self.coords.internal_grid.y) * TILE_SIZE as i32,
    )
  }
}

pub fn is_marked_for_deletion(ig: &Point<InternalGrid>) -> bool {
  ig.x < 0 || ig.y < 0 || ig.x > CHUNK_SIZE || ig.y > CHUNK_SIZE
}
