use crate::settings::{
  DEFAULT_FONT, TILE_SET_COLUMNS, TILE_SET_DEFAULT_COLUMNS, TILE_SET_DEFAULT_PATH, TILE_SET_DEFAULT_ROWS,
  TILE_SET_ROWS, TILE_SET_SAND_PATH, TILE_SIZE,
};
use bevy::asset::{AssetServer, Assets, Handle};
use bevy::math::UVec2;
use bevy::prelude::{Font, Image, Res, ResMut, TextureAtlasLayout};

#[derive(Debug, Clone)]
pub struct AssetPacks {
  pub font: Handle<Font>,
  pub default: AssetPack,
  pub sand: AssetPack,
}

#[derive(Debug, Clone)]
pub struct AssetPack {
  pub texture: Handle<Image>,
  pub texture_atlas_layout: Handle<TextureAtlasLayout>,
}

pub fn get_asset_packs(
  asset_server: &Res<AssetServer>,
  texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
) -> AssetPacks {
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

  AssetPacks {
    font: asset_server.load(DEFAULT_FONT),
    default: AssetPack {
      texture: asset_server.load(TILE_SET_DEFAULT_PATH),
      texture_atlas_layout: default_texture_atlas_layout,
    },
    sand: AssetPack {
      texture: asset_server.load(TILE_SET_SAND_PATH),
      texture_atlas_layout: texture_atlas_layout.clone(),
    },
    // grass: AssetPack {
    //   texture: asset_server.load(TILE_SET_GRASS_PATH),
    //   texture_atlas_layout: texture_atlas_layout.clone(),
    // },
  }
}
