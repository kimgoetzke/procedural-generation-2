use crate::constants::{
  DEFAULT_FONT, TILE_SET_COLUMNS, TILE_SET_DEFAULT_COLUMNS, TILE_SET_DEFAULT_PATH, TILE_SET_DEFAULT_ROWS,
  TILE_SET_FOREST_PATH, TILE_SET_GRASS_PATH, TILE_SET_ROWS, TILE_SET_SAND_PATH, TILE_SET_SHORE_PATH, TILE_SET_WATER_PATH,
  TILE_SIZE,
};
use bevy::app::{App, Plugin, Startup};
use bevy::asset::{AssetServer, Assets, Handle};
use bevy::math::UVec2;
use bevy::prelude::{Font, Image, Res, ResMut, Resource, TextureAtlasLayout};

pub struct WorldResourcesPlugin;

impl Plugin for WorldResourcesPlugin {
  fn build(&self, app: &mut App) {
    app
      .init_resource::<AssetPacks>()
      .add_systems(Startup, initialise_asset_packs_system);
  }
}

#[derive(Resource, Default, Debug, Clone)]
pub struct AssetPacks {
  pub font: Handle<Font>,
  pub default: AssetPack,
  pub water: AssetPack,
  pub shore: AssetPack,
  pub sand: AssetPack,
  pub grass: AssetPack,
  pub forest: AssetPack,
}

#[derive(Default, Debug, Clone)]
pub struct AssetPack {
  pub texture: Handle<Image>,
  pub texture_atlas_layout: Handle<TextureAtlasLayout>,
}

fn initialise_asset_packs_system(
  asset_server: Res<AssetServer>,
  mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
  mut asset_packs_resource: ResMut<AssetPacks>,
) {
  let layout = TextureAtlasLayout::from_grid(UVec2::splat(TILE_SIZE), TILE_SET_COLUMNS, TILE_SET_ROWS, None, None);
  let texture_atlas_layout = texture_atlas_layouts.add(layout);
  let default_layout = TextureAtlasLayout::from_grid(
    UVec2::splat(TILE_SIZE),
    TILE_SET_DEFAULT_COLUMNS,
    TILE_SET_DEFAULT_ROWS,
    None,
    None,
  );
  let default_texture_atlas_layout = texture_atlas_layouts.add(default_layout);

  asset_packs_resource.font = asset_server.load(DEFAULT_FONT);
  asset_packs_resource.default = AssetPack {
    texture: asset_server.load(TILE_SET_DEFAULT_PATH),
    texture_atlas_layout: default_texture_atlas_layout,
  };
  asset_packs_resource.water = AssetPack {
    texture: asset_server.load(TILE_SET_WATER_PATH),
    texture_atlas_layout: texture_atlas_layout.clone(),
  };
  asset_packs_resource.shore = AssetPack {
    texture: asset_server.load(TILE_SET_SHORE_PATH),
    texture_atlas_layout: texture_atlas_layout.clone(),
  };
  asset_packs_resource.sand = AssetPack {
    texture: asset_server.load(TILE_SET_SAND_PATH),
    texture_atlas_layout: texture_atlas_layout.clone(),
  };
  asset_packs_resource.grass = AssetPack {
    texture: asset_server.load(TILE_SET_GRASS_PATH),
    texture_atlas_layout: texture_atlas_layout.clone(),
  };
  asset_packs_resource.forest = AssetPack {
    texture: asset_server.load(TILE_SET_FOREST_PATH),
    texture_atlas_layout,
  };
}
