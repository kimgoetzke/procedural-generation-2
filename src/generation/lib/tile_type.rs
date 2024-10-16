use crate::constants::*;
use crate::generation::lib::TerrainType;
use crate::generation::resources::AssetPacksCollection;

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum TileType {
  Fill,
  InnerCornerBottomLeft,
  InnerCornerBottomRight,
  InnerCornerTopLeft,
  InnerCornerTopRight,
  OuterCornerBottomLeft,
  OuterCornerBottomRight,
  OuterCornerTopLeft,
  OuterCornerTopRight,
  TopLeftToBottomRightBridge,
  TopRightToBottomLeftBridge,
  TopFill,
  BottomFill,
  RightFill,
  LeftFill,
  Single,
  Unknown,
}

pub fn get_sprite_index_from(terrain: &TerrainType, tile_type: &TileType, asset_collection: &AssetPacksCollection) -> usize {
  match terrain {
    TerrainType::Water => get_sprite_index(&tile_type, asset_collection.water.index_offset()),
    TerrainType::Shore => get_sprite_index(&tile_type, asset_collection.shore.index_offset()),
    TerrainType::Sand => get_sprite_index(&tile_type, asset_collection.sand.index_offset()),
    TerrainType::Grass => get_sprite_index(&tile_type, asset_collection.grass.index_offset()),
    TerrainType::Forest => get_sprite_index(&tile_type, asset_collection.forest.index_offset()),
    TerrainType::Any => panic!("{}", TERRAIN_TYPE_ERROR),
  }
}

pub fn get_sprite_index(tile_type: &TileType, index_offset: usize) -> usize {
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
