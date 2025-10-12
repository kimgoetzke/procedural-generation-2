use bevy::asset::Handle;
use bevy::image::{Image, TextureAtlasLayout};

/// An asset pack groups together related assets, such as a sprite sheet and its corresponding texture atlas layout. It
/// is used to pre-initialise and bundle resources that are used repeatedly when spawning sprites.
#[derive(Debug, Clone)]
pub struct AssetPack {
  pub texture: Handle<Image>,
  pub texture_atlas_layout: Handle<TextureAtlasLayout>,
}

impl Default for AssetPack {
  fn default() -> Self {
    Self {
      texture: Handle::default(),
      texture_atlas_layout: Handle::default(),
    }
  }
}

impl AssetPack {
  pub fn new(texture: Handle<Image>, texture_atlas_layout: Handle<TextureAtlasLayout>) -> Self {
    Self {
      texture,
      texture_atlas_layout,
    }
  }
}
