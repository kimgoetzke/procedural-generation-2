use bevy::asset::Handle;
use bevy::image::{Image, TextureAtlasLayout};

/// References an image and the layout used to address its sprites.
#[derive(Debug, Clone, Default)]
pub struct SpriteSheet {
  pub texture: Handle<Image>,
  pub texture_atlas_layout: Handle<TextureAtlasLayout>,
}

impl SpriteSheet {
  /// Creates a sprite sheet from an image and layout handles.
  pub const fn new(texture: Handle<Image>, texture_atlas_layout: Handle<TextureAtlasLayout>) -> Self {
    Self {
      texture,
      texture_atlas_layout,
    }
  }
}
