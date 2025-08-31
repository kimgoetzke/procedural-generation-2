use crate::constants::*;
use crate::generation::lib::{GenerationResourcesCollection, TerrainType};
use crate::generation::resources::Climate;
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
  pub fn calculate_sprite_index(
    &self,
    terrain: &TerrainType,
    climate: &Climate,
    resources: &GenerationResourcesCollection,
  ) -> usize {
    get_sprite_index_from(&self, terrain, climate, resources)
  }
}

fn get_sprite_index_from(
  tile_type: &TileType,
  terrain: &TerrainType,
  climate: &Climate,
  resources: &GenerationResourcesCollection,
) -> usize {
  match (terrain, climate) {
    (TerrainType::Water, _) => get_sprite_index(&tile_type, resources.water.index_offset()),
    (TerrainType::Shore, _) => get_sprite_index(&tile_type, resources.shore.index_offset()),
    (TerrainType::Land1, Climate::Dry) => get_sprite_index(&tile_type, resources.land_dry_l1.index_offset()),
    (TerrainType::Land1, Climate::Moderate) => get_sprite_index(&tile_type, resources.land_moderate_l1.index_offset()),
    (TerrainType::Land1, Climate::Humid) => get_sprite_index(&tile_type, resources.land_humid_l1.index_offset()),
    (TerrainType::Land2, Climate::Dry) => get_sprite_index(&tile_type, resources.land_dry_l2.index_offset()),
    (TerrainType::Land2, Climate::Moderate) => get_sprite_index(&tile_type, resources.land_moderate_l2.index_offset()),
    (TerrainType::Land2, Climate::Humid) => get_sprite_index(&tile_type, resources.land_humid_l2.index_offset()),
    (TerrainType::Land3, Climate::Dry) => get_sprite_index(&tile_type, resources.land_dry_l3.index_offset()),
    (TerrainType::Land3, Climate::Moderate) => get_sprite_index(&tile_type, resources.land_moderate_l3.index_offset()),
    (TerrainType::Land3, Climate::Humid) => get_sprite_index(&tile_type, resources.land_humid_l3.index_offset()),
    (TerrainType::Any, _) => panic!("{}", TERRAIN_TYPE_ERROR),
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
