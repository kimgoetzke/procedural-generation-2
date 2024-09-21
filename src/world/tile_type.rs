use crate::constants::*;
use crate::world::tile::Tile;

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

pub fn get_sprite_index(tile: &Tile) -> usize {
  match tile.tile_type {
    TileType::Fill => FILL,
    TileType::InnerCornerBottomLeft => INNER_CORNER_BOTTOM_LEFT,
    TileType::InnerCornerBottomRight => INNER_CORNER_BOTTOM_RIGHT,
    TileType::InnerCornerTopLeft => INNER_CORNER_TOP_LEFT,
    TileType::InnerCornerTopRight => INNER_CORNER_TOP_RIGHT,
    TileType::OuterCornerBottomLeft => OUTER_CORNER_BOTTOM_LEFT,
    TileType::OuterCornerBottomRight => OUTER_CORNER_BOTTOM_RIGHT,
    TileType::OuterCornerTopLeft => OUTER_CORNER_TOP_LEFT,
    TileType::OuterCornerTopRight => OUTER_CORNER_TOP_RIGHT,
    TileType::TopLeftToBottomRightBridge => TOP_LEFT_TO_BOTTOM_RIGHT_BRIDGE,
    TileType::TopRightToBottomLeftBridge => TOP_RIGHT_TO_BOTTOM_LEFT_BRIDGE,
    TileType::TopFill => TOP_FILL,
    TileType::BottomFill => BOTTOM_FILL,
    TileType::RightFill => RIGHT_FILL,
    TileType::LeftFill => LEFT_FILL,
    TileType::Single => SINGLE,
    TileType::Unknown => ERROR,
  }
}
