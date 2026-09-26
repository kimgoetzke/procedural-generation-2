use crate::constants::*;
use crate::generation::model::{Climate, TerrainType, WorldResources};
use bevy::reflect::Reflect;
use strum::EnumIter;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Reflect, serde::Deserialize, EnumIter)]
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
  /// Calculates the first sprite index for this tile type.
  pub fn calculate_sprite_index(&self, terrain: &TerrainType, climate: &Climate, world_resources: &WorldResources) -> usize {
    get_sprite_index(self, world_resources.sprite_sheet_set(terrain, climate).index_offset())
  }
}

const fn get_sprite_index(tile_type: &TileType, index_offset: usize) -> usize {
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
