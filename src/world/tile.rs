use crate::settings::TILE_SIZE;
use crate::world::shared::{Coords, Point, TerrainType, TileType};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct DraftTile {
  pub coords: Coords,
  pub terrain: TerrainType,
  pub layer: i32,
}

impl DraftTile {
  pub fn new(grid_location: Point, terrain: TerrainType, layer: usize) -> Self {
    Self {
      coords: Coords::new_grid(grid_location, TILE_SIZE),
      terrain,
      layer: layer as i32,
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

  pub fn move_to_lower_terrain_layer(&mut self) {
    let new_terrain = match self.terrain {
      TerrainType::Shore => TerrainType::Water,
      TerrainType::Sand => TerrainType::Shore,
      TerrainType::Grass => TerrainType::Sand,
      TerrainType::Forest => TerrainType::Grass,
      _ => TerrainType::Water,
    };
    self.update_terrain(new_terrain);
  }

  fn update_terrain(&mut self, terrain_type: TerrainType) {
    self.terrain = terrain_type;
    self.layer = terrain_type as i32;
    self.default_sprite_index = terrain_type as usize;
  }
}
