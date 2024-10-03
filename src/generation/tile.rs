use crate::constants::{BUFFER_SIZE, CHUNK_SIZE, TILE_SIZE};
use crate::coords::{Coords, Point};
use crate::generation::draft_tile::DraftTile;
use crate::generation::terrain_type::TerrainType;
use crate::generation::tile_type::TileType;
use bevy::log::*;

/// A `Tile` represents a single tile of `TILE_SIZE` in the world. It contains information about its `Coords`,
/// `TerrainType`, `TileType`, and layer. If created from a `DraftTile`, the `layer` of a `Tile` adds the y-coordinate
/// of the world grid `Coords` to the layer from the `DraftTile` from which it was created. It also adjusts the
/// `ChunkGrid` `Coords` to account for the buffer of a `DraftChunk` i.e. it shifts the `ChunkGrid` `Coords` by the
/// `BUFFER_SIZE` to towards the top-left, allowing for the outer tiles of a `DraftChunk` to be cut off in a way that
/// the `Tile`s in the resulting `Chunk` have `ChunkGrid` `Coords` ranging from 0 to `CHUNK_SIZE`.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Tile {
  pub coords: Coords,
  pub terrain: TerrainType,
  pub layer: i32,
  pub tile_type: TileType,
}

impl Tile {
  pub fn from(draft_tile: DraftTile, tile_type: TileType) -> Self {
    let adjusted_chunk_grid = Point::new_chunk_grid(
      draft_tile.coords.chunk_grid.x - BUFFER_SIZE,
      draft_tile.coords.chunk_grid.y - BUFFER_SIZE,
    );
    let adjusted_coords = Coords::new(adjusted_chunk_grid, draft_tile.coords.world_grid);
    if !is_marked_for_deletion(&adjusted_chunk_grid) {
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
      layer: draft_tile.layer + draft_tile.coords.chunk_grid.y,
      tile_type,
    }
  }

  pub fn get_parent_chunk_world(&self) -> Point {
    Point::new_world(
      (self.coords.world_grid.x - self.coords.chunk_grid.x) * TILE_SIZE as i32,
      (self.coords.world_grid.y + self.coords.chunk_grid.y) * TILE_SIZE as i32,
    )
  }
}

pub fn is_marked_for_deletion(chunk_grid: &Point) -> bool {
  chunk_grid.x < 0 || chunk_grid.y < 0 || chunk_grid.x > CHUNK_SIZE || chunk_grid.y > CHUNK_SIZE
}
