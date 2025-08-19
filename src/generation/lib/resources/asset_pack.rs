use bevy::asset::Handle;
use bevy::image::{Image, TextureAtlasLayout};

#[derive(Debug, Clone)]
pub struct AssetPack {
  pub texture: Handle<Image>,
  pub texture_atlas_layout: Handle<TextureAtlasLayout>,
  pub index_offset: usize,
}

impl Default for AssetPack {
  fn default() -> Self {
    Self {
      texture: Handle::default(),
      texture_atlas_layout: Handle::default(),
      index_offset: 1,
    }
  }
}

impl AssetPack {
  pub fn new(texture: Handle<Image>, texture_atlas_layout: Handle<TextureAtlasLayout>) -> Self {
    Self {
      texture,
      texture_atlas_layout,
      index_offset: 1,
    }
  }
}
