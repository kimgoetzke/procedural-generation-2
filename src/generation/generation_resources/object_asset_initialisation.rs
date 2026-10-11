use crate::constants::{
  ANIMATED_OBJ_COLUMNS, ANIMATED_OBJ_ROWS, DEFAULT_OBJ_COLUMNS, DEFAULT_OBJ_ROWS, DEFAULT_OBJ_SIZE, OBJ_ANIMATED_PATH,
  OBJ_L1_DRY_PATH, OBJ_L1_HUMID_PATH, OBJ_L1_MODERATE_PATH, OBJ_L2_DRY_PATH, OBJ_L2_HUMID_PATH, OBJ_L2_MODERATE_PATH,
  OBJ_L3_DRY_PATH, OBJ_L3_HUMID_PATH, OBJ_L3_MODERATE_PATH, SETTLEMENTS_OBJ_PATH, SHORE_OBJ_PATH, STRUCTURES_OBJ_COLUMNS,
  STRUCTURES_OBJ_ROWS, TREES_DRY_OBJ_PATH, TREES_HUMID_OBJ_PATH, TREES_MODERATE_OBJ_PATH, TREES_OBJ_COLUMNS, TREES_OBJ_ROWS,
  TREES_OBJ_SIZE, WATER_OBJ_PATH,
};
use crate::generation::model::{ObjectResources, SpriteSheet, SpriteSheetSet};
use bevy::asset::{AssetServer, Assets};
use bevy::image::TextureAtlasLayout;
use bevy::platform::collections::HashSet;
use bevy::prelude::{Res, ResMut};

/// Populates sprite sheets used to render generated objects.
pub(in crate::generation::generation_resources) fn populate_object_resources(
  object_resources: &mut ObjectResources,
  asset_server: &Res<AssetServer>,
  mut layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
) {
  // Objects sprites: Trees
  let static_trees_layout = TextureAtlasLayout::from_grid(TREES_OBJ_SIZE, TREES_OBJ_COLUMNS, TREES_OBJ_ROWS, None, None);
  let static_trees_atlas_layout = layouts.add(static_trees_layout);
  object_resources.trees_dry.static_sheet =
    SpriteSheet::new(asset_server.load(TREES_DRY_OBJ_PATH), static_trees_atlas_layout.clone());
  object_resources.trees_moderate.static_sheet =
    SpriteSheet::new(asset_server.load(TREES_MODERATE_OBJ_PATH), static_trees_atlas_layout.clone());
  object_resources.trees_humid.static_sheet =
    SpriteSheet::new(asset_server.load(TREES_HUMID_OBJ_PATH), static_trees_atlas_layout);

  // Object sprites: Settlements
  let static_settlements_layout =
    TextureAtlasLayout::from_grid(DEFAULT_OBJ_SIZE, STRUCTURES_OBJ_COLUMNS, STRUCTURES_OBJ_ROWS, None, None);
  let static_settlements_atlas_layout = layouts.add(static_settlements_layout);
  object_resources.settlements.static_sheet =
    SpriteSheet::new(asset_server.load(SETTLEMENTS_OBJ_PATH), static_settlements_atlas_layout);

  // Objects sprites: Decorative terrain overlays
  object_resources.water = object_assets_static(&asset_server, &mut layouts, WATER_OBJ_PATH);
  object_resources.shore = object_assets_static(&asset_server, &mut layouts, SHORE_OBJ_PATH);
  object_resources.l1_dry = object_assets_static(&asset_server, &mut layouts, OBJ_L1_DRY_PATH);
  object_resources.l1_moderate = object_assets_static(&asset_server, &mut layouts, OBJ_L1_MODERATE_PATH);
  object_resources.l1_humid = object_assets_static(&asset_server, &mut layouts, OBJ_L1_HUMID_PATH);
  object_resources.l2_dry = object_assets_static(&asset_server, &mut layouts, OBJ_L2_DRY_PATH);
  object_resources.l2_moderate = object_assets_static(&asset_server, &mut layouts, OBJ_L2_MODERATE_PATH);
  object_resources.l2_humid = object_assets_static(&asset_server, &mut layouts, OBJ_L2_HUMID_PATH);
  object_resources.l3_dry = object_assets_static(&asset_server, &mut layouts, OBJ_L3_DRY_PATH);
  object_resources.l3_moderate = object_assets_static(&asset_server, &mut layouts, OBJ_L3_MODERATE_PATH);
  object_resources.l3_humid = object_assets_static(&asset_server, &mut layouts, OBJ_L3_HUMID_PATH);
  object_resources.animated = object_assets_animated(&asset_server, &mut layouts, OBJ_ANIMATED_PATH);
}

fn object_assets_static(
  asset_server: &Res<AssetServer>,
  layout: &mut Assets<TextureAtlasLayout>,
  tile_set_path: &str,
) -> SpriteSheetSet {
  let static_layout = TextureAtlasLayout::from_grid(DEFAULT_OBJ_SIZE, DEFAULT_OBJ_COLUMNS, DEFAULT_OBJ_ROWS, None, None);
  let static_atlas_layout = layout.add(static_layout);

  SpriteSheetSet {
    static_sheet: SpriteSheet::new(asset_server.load(tile_set_path.to_string()), static_atlas_layout),
    animated_sheet: None,
    animated_tile_types: HashSet::new(),
    index_offset: 1,
  }
}

fn object_assets_animated(
  asset_server: &Res<AssetServer>,
  layout: &mut Assets<TextureAtlasLayout>,
  tile_set_path: &str,
) -> SpriteSheetSet {
  let animated_tile_set_layout =
    TextureAtlasLayout::from_grid(DEFAULT_OBJ_SIZE, ANIMATED_OBJ_COLUMNS, ANIMATED_OBJ_ROWS, None, None);
  let atlas_layout = layout.add(animated_tile_set_layout);
  let texture = asset_server.load(tile_set_path.to_string());

  SpriteSheetSet {
    static_sheet: SpriteSheet::new(texture.clone(), atlas_layout.clone()),
    animated_sheet: Some(SpriteSheet::new(texture, atlas_layout)),
    animated_tile_types: { HashSet::new() },
    index_offset: ANIMATED_OBJ_COLUMNS as usize,
  }
}
