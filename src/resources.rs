use crate::constants::{
  DRAW_TERRAIN_SPRITES, ENABLE_TILE_DEBUGGING, GENERATE_NEIGHBOUR_CHUNKS, LAYER_POST_PROCESSING, SPAWN_UP_TO_LAYER,
};
use bevy::app::{App, Plugin};
use bevy::prelude::{Reflect, ReflectResource, Resource};
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::ReflectInspectorOptions;
use bevy_inspector_egui::InspectorOptions;

pub struct SharedResourcesPlugin;

impl Plugin for SharedResourcesPlugin {
  fn build(&self, app: &mut App) {
    app
      .init_resource::<Settings>()
      .register_type::<Settings>()
      .insert_resource(Settings::default())
      .init_resource::<GeneralGenerationSettings>()
      .register_type::<GeneralGenerationSettings>()
      .insert_resource(GeneralGenerationSettings::default())
      .init_resource::<WorldGenerationSettings>()
      .register_type::<WorldGenerationSettings>()
      .insert_resource(WorldGenerationSettings::default());
  }
}

#[derive(Resource, Reflect)]
pub struct Settings {
  pub general: GeneralGenerationSettings,
  pub world: WorldGenerationSettings,
}

impl Default for Settings {
  fn default() -> Self {
    Self {
      general: GeneralGenerationSettings::default(),
      world: WorldGenerationSettings::default(),
    }
  }
}

#[derive(Clone, Resource, Reflect, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct GeneralGenerationSettings {
  pub generate_neighbour_chunks: bool,
  pub enable_tile_debugging: bool, // Disabling massively speeds up the generation process
  pub draw_terrain_sprites: bool,
  pub layer_post_processing: bool,
  #[inspector(min = 0, max = 5, display = NumberDisplay::Slider)]
  pub spawn_up_to_layer: usize,
}

impl Default for GeneralGenerationSettings {
  fn default() -> Self {
    Self {
      generate_neighbour_chunks: GENERATE_NEIGHBOUR_CHUNKS,
      enable_tile_debugging: ENABLE_TILE_DEBUGGING,
      draw_terrain_sprites: DRAW_TERRAIN_SPRITES,
      layer_post_processing: LAYER_POST_PROCESSING,
      spawn_up_to_layer: SPAWN_UP_TO_LAYER,
    }
  }
}

#[derive(Clone, Resource, Reflect, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct WorldGenerationSettings {
  #[inspector(min = 0, max = 100, display = NumberDisplay::Slider)]
  pub noise_seed: u32,
  #[inspector(min = 0.0, max = 0.25, display = NumberDisplay::Slider)]
  pub noise_frequency: f64,
  #[inspector(min = 0., max = 3.0, display = NumberDisplay::Slider)]
  pub noise_amplitude: f64,
  #[inspector(min = -1., max = 1., display = NumberDisplay::Slider)]
  pub elevation: f64,
  #[inspector(min = 0.0, max = 2.5, display = NumberDisplay::Slider)]
  pub falloff_strength: f64,
}

impl Default for WorldGenerationSettings {
  fn default() -> Self {
    Self {
      noise_seed: 1,
      noise_frequency: 0.1, // The higher the frequency, the more detailed the terrain
      noise_amplitude: 1.,  // The higher the amplitude, the higher the peaks and lower the valleys
      elevation: 0.,        // Shifts the entire terrain up or down
      falloff_strength: 1., // The higher the falloff strength, the closer to the center the falloff begins
    }
  }
}
