use crate::constants::*;
use crate::generation::lib::{TerrainType, Tile};
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
    TerrainType::Water => get_sprite_index(&tile_type, asset_collection.water.tile_set_index_offset),
    TerrainType::Shore => get_sprite_index(&tile_type, asset_collection.shore.tile_set_index_offset),
    TerrainType::Sand => get_sprite_index(&tile_type, asset_collection.sand.tile_set_index_offset),
    TerrainType::Grass => get_sprite_index(&tile_type, asset_collection.grass.tile_set_index_offset),
    TerrainType::Forest => get_sprite_index(&tile_type, asset_collection.forest.tile_set_index_offset),
    TerrainType::Any => panic!("{}", TERRAIN_TYPE_ERROR),
  }
}

pub fn get_sprite_index(tile_type: &TileType, offset: usize) -> usize {
  match tile_type {
    TileType::Fill => FILL * offset,
    TileType::InnerCornerBottomLeft => INNER_CORNER_BOTTOM_LEFT * offset,
    TileType::InnerCornerBottomRight => INNER_CORNER_BOTTOM_RIGHT * offset,
    TileType::InnerCornerTopLeft => INNER_CORNER_TOP_LEFT * offset,
    TileType::InnerCornerTopRight => INNER_CORNER_TOP_RIGHT * offset,
    TileType::OuterCornerBottomLeft => OUTER_CORNER_BOTTOM_LEFT * offset,
    TileType::OuterCornerBottomRight => OUTER_CORNER_BOTTOM_RIGHT * offset,
    TileType::OuterCornerTopLeft => OUTER_CORNER_TOP_LEFT * offset,
    TileType::OuterCornerTopRight => OUTER_CORNER_TOP_RIGHT * offset,
    TileType::TopLeftToBottomRightBridge => TOP_LEFT_TO_BOTTOM_RIGHT_BRIDGE * offset,
    TileType::TopRightToBottomLeftBridge => TOP_RIGHT_TO_BOTTOM_LEFT_BRIDGE * offset,
    TileType::TopFill => TOP_FILL * offset,
    TileType::BottomFill => BOTTOM_FILL * offset,
    TileType::RightFill => RIGHT_FILL * offset,
    TileType::LeftFill => LEFT_FILL * offset,
    TileType::Single => SINGLE * offset,
    TileType::Unknown => ERROR * offset,
  }
}

pub fn get_animated_sprite_index(tile: &Tile) -> usize {
  match tile.tile_type {
    TileType::Fill => FILL * DEFAULT_ANIMATED_TILE_SET_COLUMNS as usize,
    TileType::InnerCornerBottomLeft => INNER_CORNER_BOTTOM_LEFT * DEFAULT_ANIMATED_TILE_SET_COLUMNS as usize,
    TileType::InnerCornerBottomRight => INNER_CORNER_BOTTOM_RIGHT * DEFAULT_ANIMATED_TILE_SET_COLUMNS as usize,
    TileType::InnerCornerTopLeft => INNER_CORNER_TOP_LEFT * DEFAULT_ANIMATED_TILE_SET_COLUMNS as usize,
    TileType::InnerCornerTopRight => INNER_CORNER_TOP_RIGHT * DEFAULT_ANIMATED_TILE_SET_COLUMNS as usize,
    TileType::OuterCornerBottomLeft => OUTER_CORNER_BOTTOM_LEFT * DEFAULT_ANIMATED_TILE_SET_COLUMNS as usize,
    TileType::OuterCornerBottomRight => OUTER_CORNER_BOTTOM_RIGHT * DEFAULT_ANIMATED_TILE_SET_COLUMNS as usize,
    TileType::OuterCornerTopLeft => OUTER_CORNER_TOP_LEFT * DEFAULT_ANIMATED_TILE_SET_COLUMNS as usize,
    TileType::OuterCornerTopRight => OUTER_CORNER_TOP_RIGHT * DEFAULT_ANIMATED_TILE_SET_COLUMNS as usize,
    TileType::TopLeftToBottomRightBridge => TOP_LEFT_TO_BOTTOM_RIGHT_BRIDGE * DEFAULT_ANIMATED_TILE_SET_COLUMNS as usize,
    TileType::TopRightToBottomLeftBridge => TOP_RIGHT_TO_BOTTOM_LEFT_BRIDGE * DEFAULT_ANIMATED_TILE_SET_COLUMNS as usize,
    TileType::TopFill => TOP_FILL * DEFAULT_ANIMATED_TILE_SET_COLUMNS as usize,
    TileType::BottomFill => BOTTOM_FILL * DEFAULT_ANIMATED_TILE_SET_COLUMNS as usize,
    TileType::RightFill => RIGHT_FILL * DEFAULT_ANIMATED_TILE_SET_COLUMNS as usize,
    TileType::LeftFill => LEFT_FILL * DEFAULT_ANIMATED_TILE_SET_COLUMNS as usize,
    TileType::Single => SINGLE * DEFAULT_ANIMATED_TILE_SET_COLUMNS as usize,
    TileType::Unknown => ERROR * DEFAULT_ANIMATED_TILE_SET_COLUMNS as usize,
  }
}
