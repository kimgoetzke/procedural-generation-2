use crate::constants::{
  ANIMATED_TILE_SET_COLUMNS, PLACEHOLDER_TILE_SET_COLUMNS, PLACEHOLDER_TILE_SET_ROWS, STATIC_TILE_SET_COLUMNS,
  TILE_SET_ROWS, TILE_SIZE, TS_LAND_DRY_L1_PATH, TS_LAND_DRY_L2_PATH, TS_LAND_DRY_L3_PATH, TS_LAND_HUMID_L1_PATH,
  TS_LAND_HUMID_L2_PATH, TS_LAND_HUMID_L3_PATH, TS_LAND_MODERATE_L1_PATH, TS_LAND_MODERATE_L2_PATH,
  TS_LAND_MODERATE_L3_PATH, TS_PLACEHOLDER_PATH, TS_SHORE_PATH, TS_WATER_PATH,
};
use crate::generation::model::{AssetCollection, AssetPack, GenerationResourcesCollection, TileType};
use bevy::asset::{AssetServer, Assets};
use bevy::image::TextureAtlasLayout;
use bevy::math::UVec2;
use bevy::platform::collections::HashSet;
use bevy::prelude::{Res, ResMut};

/// Populates all terrain related resources of the provided [`GenerationResourcesCollection`] by loading the relevant
/// assets and creating [`AssetCollection`]s for each terrain layer/type.
pub(crate) fn populate_terrain_assets(
  grc: &mut ResMut<GenerationResourcesCollection>,
  asset_server: &Res<AssetServer>,
  mut layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
) {
  // Placeholder tile set
  let default_layout = TextureAtlasLayout::from_grid(
    UVec2::splat(TILE_SIZE),
    PLACEHOLDER_TILE_SET_COLUMNS,
    PLACEHOLDER_TILE_SET_ROWS,
    None,
    None,
  );
  let default_texture_atlas_layout = layouts.add(default_layout);
  grc.placeholder = AssetPack::new(asset_server.load(TS_PLACEHOLDER_PATH), default_texture_atlas_layout);

  // Detailed tile sets
  grc.water = tile_set_animated(&asset_server, &mut layouts, TS_WATER_PATH, true, ANIMATED_TILE_SET_COLUMNS);
  grc.shore = tile_set_animated(&asset_server, &mut layouts, TS_SHORE_PATH, true, ANIMATED_TILE_SET_COLUMNS);
  grc.land_dry_l1 = tile_set_animated(
    &asset_server,
    &mut layouts,
    TS_LAND_DRY_L1_PATH,
    false,
    ANIMATED_TILE_SET_COLUMNS,
  );
  grc.land_dry_l2 = tile_set_static(&asset_server, &mut layouts, TS_LAND_DRY_L2_PATH);
  grc.land_dry_l3 = tile_set_static(&asset_server, &mut layouts, TS_LAND_DRY_L3_PATH);
  grc.land_moderate_l1 = tile_set_animated(
    &asset_server,
    &mut layouts,
    TS_LAND_MODERATE_L1_PATH,
    false,
    ANIMATED_TILE_SET_COLUMNS,
  );
  grc.land_moderate_l2 = tile_set_static(&asset_server, &mut layouts, TS_LAND_MODERATE_L2_PATH);
  grc.land_moderate_l3 = tile_set_static(&asset_server, &mut layouts, TS_LAND_MODERATE_L3_PATH);
  grc.land_humid_l1 = tile_set_animated(
    &asset_server,
    &mut layouts,
    TS_LAND_HUMID_L1_PATH,
    false,
    ANIMATED_TILE_SET_COLUMNS,
  );
  grc.land_humid_l2 = tile_set_static(&asset_server, &mut layouts, TS_LAND_HUMID_L2_PATH);
  grc.land_humid_l3 = tile_set_static(&asset_server, &mut layouts, TS_LAND_HUMID_L3_PATH);
}

fn tile_set_static(
  asset_server: &Res<AssetServer>,
  layout: &mut Assets<TextureAtlasLayout>,
  tile_set_path: &str,
) -> AssetCollection {
  let static_layout =
    TextureAtlasLayout::from_grid(UVec2::splat(TILE_SIZE), STATIC_TILE_SET_COLUMNS, TILE_SET_ROWS, None, None);
  let texture_atlas_layout = layout.add(static_layout);

  AssetCollection {
    stat: AssetPack::new(asset_server.load(tile_set_path.to_string()), texture_atlas_layout),
    anim: None,
    animated_tile_types: HashSet::new(),
    index_offset: 1,
  }
}

fn tile_set_animated(
  asset_server: &Res<AssetServer>,
  layout: &mut Assets<TextureAtlasLayout>,
  tile_set_path: &str,
  is_fill_animated: bool,
  columns: u32,
) -> AssetCollection {
  let animated_tile_set_layout = TextureAtlasLayout::from_grid(UVec2::splat(TILE_SIZE), columns, TILE_SET_ROWS, None, None);
  let atlas_layout = layout.add(animated_tile_set_layout);
  let texture = asset_server.load(tile_set_path.to_string());

  AssetCollection {
    stat: AssetPack::new(texture.clone(), atlas_layout.clone()),
    anim: Some(AssetPack::new(texture, atlas_layout)),
    animated_tile_types: {
      let mut tile_types_set = HashSet::from([
        TileType::InnerCornerBottomLeft,
        TileType::InnerCornerBottomRight,
        TileType::InnerCornerTopLeft,
        TileType::InnerCornerTopRight,
        TileType::OuterCornerBottomLeft,
        TileType::OuterCornerBottomRight,
        TileType::OuterCornerTopLeft,
        TileType::OuterCornerTopRight,
        TileType::TopLeftToBottomRightBridge,
        TileType::TopRightToBottomLeftBridge,
        TileType::TopFill,
        TileType::BottomFill,
        TileType::RightFill,
        TileType::LeftFill,
        TileType::Single,
      ]);
      if is_fill_animated {
        tile_types_set.insert(TileType::Fill);
      }

      tile_types_set
    },
    index_offset: columns as usize,
  }
}
