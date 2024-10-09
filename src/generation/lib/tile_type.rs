use crate::constants::*;
use crate::generation::lib::Tile;

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

pub fn get_static_sprite_index(tile: &Tile) -> usize {
  match tile.tile_type {
    TileType::Fill => STATIC_FILL,
    TileType::InnerCornerBottomLeft => STATIC_INNER_CORNER_BOTTOM_LEFT,
    TileType::InnerCornerBottomRight => STATIC_INNER_CORNER_BOTTOM_RIGHT,
    TileType::InnerCornerTopLeft => STATIC_INNER_CORNER_TOP_LEFT,
    TileType::InnerCornerTopRight => STATIC_INNER_CORNER_TOP_RIGHT,
    TileType::OuterCornerBottomLeft => STATIC_OUTER_CORNER_BOTTOM_LEFT,
    TileType::OuterCornerBottomRight => STATIC_OUTER_CORNER_BOTTOM_RIGHT,
    TileType::OuterCornerTopLeft => STATIC_OUTER_CORNER_TOP_LEFT,
    TileType::OuterCornerTopRight => STATIC_OUTER_CORNER_TOP_RIGHT,
    TileType::TopLeftToBottomRightBridge => STATIC_TOP_LEFT_TO_BOTTOM_RIGHT_BRIDGE,
    TileType::TopRightToBottomLeftBridge => STATIC_TOP_RIGHT_TO_BOTTOM_LEFT_BRIDGE,
    TileType::TopFill => STATIC_TOP_FILL,
    TileType::BottomFill => STATIC_BOTTOM_FILL,
    TileType::RightFill => STATIC_RIGHT_FILL,
    TileType::LeftFill => STATIC_LEFT_FILL,
    TileType::Single => STATIC_SINGLE,
    TileType::Unknown => STATIC_ERROR,
  }
}

pub fn get_animated_sprite_index(tile: &Tile) -> usize {
  match tile.tile_type {
    TileType::Fill => ANIMATED_FILL,
    TileType::InnerCornerBottomLeft => ANIMATED_INNER_CORNER_BOTTOM_LEFT,
    TileType::InnerCornerBottomRight => ANIMATED_INNER_CORNER_BOTTOM_RIGHT,
    TileType::InnerCornerTopLeft => ANIMATED_INNER_CORNER_TOP_LEFT,
    TileType::InnerCornerTopRight => ANIMATED_INNER_CORNER_TOP_RIGHT,
    TileType::OuterCornerBottomLeft => ANIMATED_OUTER_CORNER_BOTTOM_LEFT,
    TileType::OuterCornerBottomRight => ANIMATED_OUTER_CORNER_BOTTOM_RIGHT,
    TileType::OuterCornerTopLeft => ANIMATED_OUTER_CORNER_TOP_LEFT,
    TileType::OuterCornerTopRight => ANIMATED_OUTER_CORNER_TOP_RIGHT,
    TileType::TopLeftToBottomRightBridge => ANIMATED_TOP_LEFT_TO_BOTTOM_RIGHT_BRIDGE,
    TileType::TopRightToBottomLeftBridge => ANIMATED_TOP_RIGHT_TO_BOTTOM_LEFT_BRIDGE,
    TileType::TopFill => ANIMATED_TOP_FILL,
    TileType::BottomFill => ANIMATED_BOTTOM_FILL,
    TileType::RightFill => ANIMATED_RIGHT_FILL,
    TileType::LeftFill => ANIMATED_LEFT_FILL,
    TileType::Single => ANIMATED_SINGLE,
    TileType::Unknown => ANIMATED_ERROR,
  }
}
