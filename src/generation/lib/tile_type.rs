use crate::constants::*;
use crate::generation::lib::TerrainType;
use crate::generation::resources::GenerationResourcesCollection;
use bevy::reflect::Reflect;

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Reflect, serde::Deserialize)]
pub enum TileType {
  Fill,
  InnerCornerTopRight,
  InnerCornerBottomRight,
  InnerCornerBottomLeft,
  InnerCornerTopLeft,
  OuterCornerTopRight,
  OuterCornerBottomRight,
  OuterCornerBottomLeft,
  OuterCornerTopLeft,
  TopRightToBottomLeftBridge,
  TopLeftToBottomRightBridge,
  TopFill,
  RightFill,
  BottomFill,
  LeftFill,
  Single,
  Unknown,
}

impl TileType {
  pub fn get_sprite_index(&self, index_offset: usize) -> usize {
    get_sprite_index(self, index_offset)
  }

  pub fn calculate_sprite_index(&self, terrain: &TerrainType, resources: &GenerationResourcesCollection) -> usize {
    get_sprite_index_from(&self, terrain, resources)
  }
}

fn get_sprite_index_from(tile_type: &TileType, terrain: &TerrainType, resources: &GenerationResourcesCollection) -> usize {
  match terrain {
    TerrainType::Water => get_sprite_index(&tile_type, resources.water.index_offset()),
    TerrainType::Shore => get_sprite_index(&tile_type, resources.shore.index_offset()),
    TerrainType::Sand => get_sprite_index(&tile_type, resources.sand.index_offset()),
    TerrainType::Grass => get_sprite_index(&tile_type, resources.grass.index_offset()),
    TerrainType::Forest => get_sprite_index(&tile_type, resources.forest.index_offset()),
    TerrainType::Any => panic!("{}", TERRAIN_TYPE_ERROR),
  }
}

fn get_sprite_index(tile_type: &TileType, index_offset: usize) -> usize {
  match tile_type {
    TileType::Fill => FILL * index_offset,
    TileType::InnerCornerBottomLeft => INNER_CORNER_BOTTOM_LEFT * index_offset,
    TileType::InnerCornerBottomRight => INNER_CORNER_BOTTOM_RIGHT * index_offset,
    TileType::InnerCornerTopLeft => INNER_CORNER_TOP_LEFT * index_offset,
    TileType::InnerCornerTopRight => INNER_CORNER_TOP_RIGHT * index_offset,
    TileType::OuterCornerBottomLeft => OUTER_CORNER_BOTTOM_LEFT * index_offset,
    TileType::OuterCornerBottomRight => OUTER_CORNER_BOTTOM_RIGHT * index_offset,
    TileType::OuterCornerTopLeft => OUTER_CORNER_TOP_LEFT * index_offset,
    TileType::OuterCornerTopRight => OUTER_CORNER_TOP_RIGHT * index_offset,
    TileType::TopLeftToBottomRightBridge => TOP_LEFT_TO_BOTTOM_RIGHT_BRIDGE * index_offset,
    TileType::TopRightToBottomLeftBridge => TOP_RIGHT_TO_BOTTOM_LEFT_BRIDGE * index_offset,
    TileType::TopFill => TOP_FILL * index_offset,
    TileType::BottomFill => BOTTOM_FILL * index_offset,
    TileType::RightFill => RIGHT_FILL * index_offset,
    TileType::LeftFill => LEFT_FILL * index_offset,
    TileType::Single => SINGLE * index_offset,
    TileType::Unknown => ERROR * index_offset,
  }
}
