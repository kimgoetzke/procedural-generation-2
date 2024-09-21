use crate::coords::{Coords, Point};
use crate::world::terrain_type::TerrainType;
use crate::world::tile_type::TileType;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct DraftTile {
  pub coords: Coords,
  pub terrain: TerrainType,
  pub layer: i32,
}

impl DraftTile {
  pub fn new(chunk_grid: Point, tile_grid: Point, terrain: TerrainType) -> Self {
    Self {
      coords: Coords::new(chunk_grid, tile_grid),
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

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Tile {
  pub coords: Coords,
  pub terrain: TerrainType,
  pub default_sprite_index: usize,
  pub layer: i32,
  pub tile_type: TileType,
}

impl Tile {
  pub fn from(draft_tile: DraftTile, tile_type: TileType) -> Self {
    Self {
      coords: draft_tile.coords,
      terrain: draft_tile.terrain,
      default_sprite_index: draft_tile.layer as usize,
      layer: draft_tile.layer,
      tile_type,
    }
  }
}
