use crate::coords::point::{ChunkGrid, WorldGrid};
use crate::coords::{Coords, Point};
use crate::generation::terrain_type::TerrainType;

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
  pub fn new(chunk_grid: Point<ChunkGrid>, world_grid: Point<WorldGrid>, terrain: TerrainType) -> Self {
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
