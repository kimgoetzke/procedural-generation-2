use crate::constants::{DRAW_SPRITES, PERMIT_TILE_LAYER_ADJUSTMENTS, SPAWN_TILE_DEBUG_INFO};
use bevy::app::{App, Plugin};
use bevy::prelude::{Reflect, ReflectResource, Resource};
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::ReflectInspectorOptions;
use bevy_inspector_egui::InspectorOptions;

pub struct SharedResourcesPlugin;

impl Plugin for SharedResourcesPlugin {
  fn build(&self, app: &mut App) {
    app
      .register_type::<ShowDebugInfo>()
      .insert_resource(ShowDebugInfo::off())
      .init_resource::<Settings>()
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
  pub spawn_tile_debug_info: bool, // Disabling massively speeds up the generation process
  pub draw_terrain_sprites: bool,
  pub permit_tile_layer_adjustments: bool,
  pub world_gen: WorldGenerationSettings,
}

impl Default for Settings {
  fn default() -> Self {
    Self {
      spawn_tile_debug_info: SPAWN_TILE_DEBUG_INFO,
      draw_terrain_sprites: DRAW_SPRITES,
      permit_tile_layer_adjustments: PERMIT_TILE_LAYER_ADJUSTMENTS,
      world_gen: WorldGenerationSettings::default(),
    }
  }
}

#[derive(Resource, Reflect, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct WorldGenerationSettings {
  #[inspector(min = 0, max = 100, display = NumberDisplay::Slider)]
  pub noise_seed: u32,
  #[inspector(min = 0.0, max = 5.0, display = NumberDisplay::Slider)]
  pub noise_frequency: f64,
  #[inspector(min = 0.0, max = 2.0, display = NumberDisplay::Slider)]
  pub noise_scale_factor: f64,
  #[inspector(min = 0.0, max = 30.0, display = NumberDisplay::Slider)]
  pub falloff_strength: f64,
}

impl Default for WorldGenerationSettings {
  fn default() -> Self {
    Self {
      noise_seed: 1,
      noise_frequency: 0.1,
      noise_scale_factor: 1.,
      falloff_strength: 1., // Higher values should result in more water at edges
    }
  }
}
