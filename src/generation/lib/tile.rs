use crate::constants::{BUFFER_SIZE, CHUNK_SIZE, TILE_SIZE};
use crate::coords::point::{InternalGrid, World};
use crate::coords::{Coords, Point};
use crate::generation::lib::debug_data::DebugData;
use crate::generation::lib::{DraftTile, TerrainType, TileType};
use crate::generation::resources::Climate;
use bevy::log::*;
use bevy::reflect::Reflect;
use std::fmt;

/// A `Tile` represents a single tile of `TILE_SIZE` in the world. It contains information about its `Coords`,
/// `TerrainType`, `TileType`, and layer. If created from a `DraftTile`, the `layer` of a `Tile` adds the y-coordinate
/// of the world grid `Coords` to the layer from the `DraftTile` from which it was created. It also adjusts the
/// `InternalGrid` `Coords` to account for the buffer of a "draft chunk" i.e. it shifts the `InternalGrid` `Coords` by the
/// `BUFFER_SIZE` to towards the top-left, allowing for the outer tiles of the "draft chunk" to be cut off in a way that
/// the `Tile`s in the resulting `Chunk` have `InternalGrid` `Coords` ranging from 0 to `CHUNK_SIZE`.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Reflect)]
pub struct Tile {
  #[reflect(ignore)]
  pub coords: Coords,
  pub terrain: TerrainType,
  pub layer: i32,
  pub climate: Climate,
  pub tile_type: TileType,
  pub debug_data: DebugData,
}

impl Tile {
  pub fn from(draft_tile: DraftTile, tile_type: TileType) -> Self {
    let adjusted_ig = Point::new_internal_grid(
      draft_tile.coords.internal_grid.x - BUFFER_SIZE,
      draft_tile.coords.internal_grid.y - BUFFER_SIZE,
    );
    let adjusted_coords = Coords::new_for_tile(adjusted_ig, draft_tile.coords.tile_grid);
    if !is_marked_for_deletion(&adjusted_ig) {
      trace!(
        "Converting: DraftTile {:?} => {:?} {:?} tile {:?}",
        draft_tile.coords, tile_type, draft_tile.terrain, adjusted_coords,
      );
    }
    Self {
      coords: adjusted_coords,
      terrain: draft_tile.terrain,
      layer: draft_tile.layer + draft_tile.coords.internal_grid.y,
      climate: draft_tile.climate,
      tile_type,
      debug_data: draft_tile.debug_data,
    }
  }

  pub fn get_parent_chunk_w(&self) -> Point<World> {
    Point::new_world(
      (self.coords.tile_grid.x - self.coords.internal_grid.x) * TILE_SIZE as i32,
      (self.coords.tile_grid.y + self.coords.internal_grid.y) * TILE_SIZE as i32,
    )
  }

  pub fn lower_terrain_by_one(&mut self, tile_type: TileType) {
    self.terrain = TerrainType::from(self.terrain as usize - 1);
    self.tile_type = tile_type;
  }
}

pub fn is_marked_for_deletion(ig: &Point<InternalGrid>) -> bool {
  ig.x < 0 || ig.y < 0 || ig.x > CHUNK_SIZE || ig.y > CHUNK_SIZE
}

impl fmt::Debug for Tile {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Tile")
      .field("coords", &self.coords)
      .field("terrain", &self.terrain)
      .field("climate", &self.climate)
      .field("tile_type", &self.tile_type)
      .finish()
  }
}
