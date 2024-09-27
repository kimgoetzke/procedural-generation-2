use crate::constants::*;
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
      .init_resource::<ObjectGenerationSettings>()
      .register_type::<ObjectGenerationSettings>()
      .insert_resource(ObjectGenerationSettings::default())
      .init_resource::<WorldGenerationSettings>()
      .register_type::<WorldGenerationSettings>()
      .insert_resource(WorldGenerationSettings::default());
  }
}

#[derive(Resource, Reflect, Clone, Copy)]
pub struct Settings {
  pub general: GeneralGenerationSettings,
  pub world: WorldGenerationSettings,
  pub object: ObjectGenerationSettings,
}

impl Default for Settings {
  fn default() -> Self {
    Self {
      general: GeneralGenerationSettings::default(),
      world: WorldGenerationSettings::default(),
      object: ObjectGenerationSettings::default(),
    }
  }
}

#[derive(Resource, Reflect, InspectorOptions, Clone, Copy)]
#[reflect(Resource, InspectorOptions)]
pub struct GeneralGenerationSettings {
  pub generate_neighbour_chunks: bool,
  pub enable_tile_debugging: bool, // Disabling massively speeds up the generation process
  pub draw_terrain_sprites: bool,
  #[inspector(min = 0, max = 5, display = NumberDisplay::Slider)]
  pub spawn_up_to_layer: usize,
}

impl Default for GeneralGenerationSettings {
  fn default() -> Self {
    Self {
      generate_neighbour_chunks: GENERATE_NEIGHBOUR_CHUNKS,
      enable_tile_debugging: ENABLE_TILE_DEBUGGING,
      draw_terrain_sprites: DRAW_TERRAIN_SPRITES,
      spawn_up_to_layer: SPAWN_UP_TO_LAYER,
    }
  }
}

#[derive(Resource, Reflect, InspectorOptions, Clone, Copy)]
#[reflect(Resource, InspectorOptions)]
pub struct WorldGenerationSettings {
  /// The seed for the noise function. A parameter of `BasicMulti`. Allows for the same terrain to be generated i.e.
  /// the same seed will always generate the exact same terrain.
  #[inspector(min = 0, max = 100, display = NumberDisplay::Slider)]
  pub noise_seed: u32,
  /// The amount of detail: The higher the octaves, the more detailed the terrain. A parameter of `BasicMulti`.
  #[inspector(min = 0, max = 12, display = NumberDisplay::Slider)]
  pub noise_octaves: usize,
  #[inspector(min = 0.0, max = 0.25, display = NumberDisplay::Slider)]
  /// The scale: the higher the frequency, the smaller the terrain features. A parameter of `BasicMulti`.
  pub noise_frequency: f64,
  /// The abruptness of changes in terrain: The higher the persistence, the rougher the terrain. A parameter of
  /// `BasicMulti`.
  #[inspector(min = 0., max = 2., display = NumberDisplay::Slider)]
  pub noise_persistence: f64,
  #[inspector(min = 0., max = 10., display = NumberDisplay::Slider)]
  /// The higher the amplitude, the more extreme the terrain. Similar to `noise_persistence` but applies to the entire
  /// output of the noise function equally. A custom parameter that is not part of `BasicMulti`.
  pub noise_amplitude: f64,
  #[inspector(min = -1., max = 1., display = NumberDisplay::Slider)]
  /// Shifts the entire terrain up or down.
  pub elevation: f64,
  /// Force the outside of the `Chunk` to become the lowest `TerrainType`. The higher the falloff strength, the closer
  /// to the center of the `Chunk` the falloff begins. Basically turns `Chunk`s into an islands. Can generate unpleasant
  /// visual artifacts if set to a very low non-zero value. Don't use it if you want `Chunk`s to be connected at a
  /// higher `TerrainType`.
  #[inspector(min = 0.0, max = 2.5, display = NumberDisplay::Slider)]
  pub falloff_strength: f64,
}

impl Default for WorldGenerationSettings {
  fn default() -> Self {
    Self {
      noise_seed: NOISE_SEED,
      noise_octaves: NOISE_OCTAVES,
      noise_frequency: NOISE_FREQUENCY,
      noise_persistence: NOISE_PERSISTENCE,
      noise_amplitude: NOISE_AMPLITUDE,
      elevation: NOISE_ELEVATION,
      falloff_strength: FALLOFF_STRENGTH,
    }
  }
}

#[derive(Resource, Reflect, InspectorOptions, Clone, Copy)]
#[reflect(Resource, InspectorOptions)]
pub struct ObjectGenerationSettings {
  pub object_generation: bool,
  #[inspector(min = 0., max = 1., display = NumberDisplay::Slider)]
  pub tree_density: f64,
}

impl Default for ObjectGenerationSettings {
  fn default() -> Self {
    Self {
      object_generation: OBJECT_GENERATION,
      tree_density: TREE_DENSITY,
    }
  }
}
