use crate::settings::{DRAW_SPRITES, PERMIT_TILE_LAYER_ADJUSTMENTS};
use bevy::app::{App, Plugin};
use bevy::prelude::{Reflect, Resource};

pub struct SharedResourcesPlugin;

impl Plugin for SharedResourcesPlugin {
  fn build(&self, app: &mut App) {
    app
      .register_type::<ShowDebugInfo>()
      .insert_resource(ShowDebugInfo::off())
      .register_type::<Settings>()
      .insert_resource(Settings::default());
  }
}

#[derive(Resource, Default, Reflect)]
pub(crate) struct ShowDebugInfo {
  pub is_on: bool,
}

impl ShowDebugInfo {
  fn off() -> Self {
    Self { is_on: false }
  }
}

#[derive(Resource, Reflect)]
pub struct Settings {
  pub draw_terrain_sprites: bool,
  pub permit_tile_layer_adjustments: bool,
}

impl Default for Settings {
  fn default() -> Self {
    Self {
      draw_terrain_sprites: DRAW_SPRITES,
      permit_tile_layer_adjustments: PERMIT_TILE_LAYER_ADJUSTMENTS,
    }
  }
}
