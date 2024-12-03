use crate::coords::point::{InternalGrid, TileGrid};
use crate::coords::{Coords, Point};
use crate::generation::lib::debug_data::DebugData;
use crate::generation::lib::terrain_type::TerrainType;
use crate::generation::resources::Climate;

/// A `DraftTile` contains the key information to generate a `Tile` and is therefore only an intermediate
/// representation. While the `Coords` and `TerrainType` of a tile will remain the same after the conversion, the
/// `layer` will be modified when creating a `Tile` from a `DraftTile` by adding the y-coordinate of the world grid
/// `Coords` to the layer.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct DraftTile {
  pub coords: Coords,
  pub terrain: TerrainType,
  pub layer: i32,
  pub climate: Climate,
  pub debug_data: DebugData,
}

impl DraftTile {
  pub fn new(
    ig: Point<InternalGrid>,
    tg: Point<TileGrid>,
    terrain: TerrainType,
    climate: Climate,
    debug_data: DebugData,
  ) -> Self {
    Self {
      coords: Coords::new_for_tile(ig, tg),
      terrain,
      climate,
      layer: terrain as i32,
      debug_data,
    }
  }

  pub fn clone_with_modified_terrain(&self, terrain: TerrainType) -> Self {
    Self {
      coords: self.coords.clone(),
      terrain,
      climate: self.climate,
      layer: terrain as i32,
      debug_data: self.debug_data.clone(),
    }
  }
}
