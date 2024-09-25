use crate::constants::{BUFFER_SIZE, TILE_SIZE};
use crate::coords::{Coords, Point};
use crate::world::terrain_type::TerrainType;
use crate::world::tile_type::TileType;

/// A `DraftTile` contains the key information to generate a `Tile` and is therefore only an intermediate
/// representation. While the `Coords` and `TerrainType` of a tile will remain the same after the conversion, the
/// `layer` will be modified when creating a `Tile` from a `DraftTile` by adding the y-coordinate of the world grid
/// `Coords` to the layer.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct DraftTile {
  pub coords: Coords,
  pub terrain: TerrainType,
  pub layer: i32,
}

impl DraftTile {
  pub fn new(chunk_grid: Point, world_grid: Point, terrain: TerrainType) -> Self {
    Self {
      coords: Coords::new(chunk_grid, world_grid),
      terrain,
      layer: terrain as i32,
    }
  }

  pub fn clone_with_modified_terrain(&self, terrain: TerrainType) -> Self {
    Self {
      coords: self.coords.clone(),
      terrain,
      layer: terrain as i32,
    }
  }
}

/// A `Tile` represents a single tile in the world. It contains information about its `Coords`, `TerrainType`,
/// `TileType`, and layer. Importantly, the `layer` of a `Tile` adds the y-coordinate of the world grid `Coords` to
/// the layer from the `DraftTile` from which it was created.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Tile {
  pub coords: Coords,
  pub terrain: TerrainType,
  pub layer: i32,
  pub tile_type: TileType,
}

impl Tile {
  pub fn from(draft_tile: DraftTile, tile_type: TileType) -> Self {
    let adjusted_chunk_grid = Point::new(
      draft_tile.coords.chunk_grid.x - BUFFER_SIZE,
      draft_tile.coords.chunk_grid.y - BUFFER_SIZE,
    );
    let adjusted_coords = Coords::new(adjusted_chunk_grid, draft_tile.coords.world_grid);
    Self {
      coords: adjusted_coords,
      terrain: draft_tile.terrain,
      layer: draft_tile.layer + draft_tile.coords.chunk_grid.y,
      tile_type,
    }
  }

  pub fn get_parent_chunk_world_point(&self) -> Point {
    Point {
      x: (self.coords.world_grid.x - self.coords.chunk_grid.x) * TILE_SIZE as i32,
      y: (self.coords.world_grid.y - self.coords.chunk_grid.y) * TILE_SIZE as i32,
    }
  }
}
