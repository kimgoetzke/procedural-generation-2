use crate::constants::*;
use bevy::app::{App, Plugin};
use bevy::prelude::{Reflect, ReflectResource, Resource};
use bevy_inspector_egui::InspectorOptions;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::ReflectInspectorOptions;

/// A plugin that registers and initialises [`Settings`] and the resources that it encompasses.
pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
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
      .insert_resource(WorldGenerationSettings::default())
      .init_resource::<GenerationMetadataSettings>()
      .register_type::<GenerationMetadataSettings>()
      .insert_resource(GenerationMetadataSettings::default());
  }
}

#[derive(Resource, Reflect, Clone, Copy, Default)]
pub struct Settings {
  pub general: GeneralGenerationSettings,
  pub metadata: GenerationMetadataSettings,
  pub world: WorldGenerationSettings,
  pub object: ObjectGenerationSettings,
}

#[derive(Resource, Reflect, InspectorOptions, Clone, Copy)]
#[reflect(Resource, InspectorOptions)]
pub struct GeneralGenerationSettings {
  /// Whether to display performance statistics for diagnostics in the top-right corner of the screen, such as FPS and
  /// frame time.
  pub display_diagnostics: bool,
  /// Whether to generate the 8 neighbouring chunks around the current chunk. If set to `false`, only the current
  /// chunk will be generated. Disabling this is only useful for debugging purposes.
  /// Whether to draw helper gizmos in the world, such as a grid indicating chunk boundaries and tile boundaries for
  /// the current chunk. Enabling this is only useful for debugging purposes.
  pub draw_gizmos: bool,
  /// Whether to enable tile debugging: when enabled, clicking on a tile will print its metadata to the console.
  /// Enabling this is only useful for debugging purposes.
  pub enable_tile_debugging: bool,
  /// Whether to generate the 8 neighbouring chunks around the current chunk. If set to `false`, only the current
  /// chunk will be generated. Disabling this is only useful for debugging purposes.
  pub generate_neighbour_chunks: bool,
  /// Whether to draw terrain sprites. If set to `false`, only placeholder sprites indicating the terrain layer will be
  /// drawn. Disabling this will cause `animate_terrain_sprites` to be ignored. Disabling this is only useful for
  /// debugging purposes.
  pub draw_terrain_sprites: bool,
  /// Whether to animate terrain sprites (if the resources collection contains animated sprite sheets for a given
  /// terrain and tile type combination). If set to `false`, terrain sprites will be drawn as static images. If set to
  /// `true`, water sprites, for example, will be animated.
  pub animate_terrain_sprites: bool,
  /// The lowest terrain layer for which terrain meshes will be spawned. See [`crate::generation::model::TerrainType`].
  /// Setting this to a value higher than `0` is only useful for debugging purposes.
  #[inspector(min = 0, max = 4, display = NumberDisplay::Slider)]
  pub spawn_from_layer: usize,
  /// The highest terrain layer for which terrain meshes will be spawned. See [`crate::generation::model::TerrainType`].
  /// Setting this to a value lower than the maximum is only useful for debugging purposes.
  #[inspector(min = 0, max = 4, display = NumberDisplay::Slider)]
  pub spawn_up_to_layer: usize,
  /// Whether to enable world pruning: when enabled, chunks that are far away from the current chunk will be despawned
  /// for performance reasons. Disabling this will cause all generated chunks to remain in the world forever.
  pub enable_world_pruning: bool,
  #[inspector(min = 0., max = 2., display = NumberDisplay::Slider)]
  pub camera_default_zoom: f32,
}

impl Default for GeneralGenerationSettings {
  fn default() -> Self {
    Self {
      display_diagnostics: DISPLAY_DIAGNOSTICS,
      draw_gizmos: DRAW_GIZMOS,
      enable_tile_debugging: ENABLE_TILE_DEBUGGING,
      generate_neighbour_chunks: GENERATE_NEIGHBOUR_CHUNKS,
      draw_terrain_sprites: DRAW_TERRAIN_SPRITES,
      animate_terrain_sprites: ANIMATE_TERRAIN_SPRITES,
      spawn_from_layer: SPAWN_FROM_LAYER,
      spawn_up_to_layer: SPAWN_UP_TO_LAYER,
      enable_world_pruning: ENABLE_WORLD_PRUNING,
      camera_default_zoom: CAMERA_DEFAULT_ZOOM,
    }
  }
}

#[derive(Resource, Reflect, InspectorOptions, Clone, Copy)]
#[reflect(Resource, InspectorOptions)]
pub struct GenerationMetadataSettings {
  /// The total elevation change within a chunk. The higher the value, the faster (i.e. over a distance of fewer
  /// chunks) the terrain oscillates between the highest and lowest terrain layers.
  #[inspector(min = 0.0, max = 0.2, display = NumberDisplay::Slider)]
  pub elevation_chunk_step_size: f64,
  /// Shifts the ranges generated for the elevation metadata up or down. The higher the value the more the ranges
  /// will shift into negative values which causes lower terrain layers to be generated for chunks with the lowest
  /// ranges and less high terrain layers for chunks with the higher ranges.
  #[inspector(min = -1.0, max = 1.0, display = NumberDisplay::Slider)]
  pub elevation_offset: f64,
  /// The scale of the noise map generated for the biome metadata: the higher the frequency, the smaller the terrain
  /// features. A parameter of [`noise::BasicMulti<noise::Perlin>`].
  #[inspector(min = 0.0, max = 0.25, display = NumberDisplay::Slider)]
  pub biome_noise_frequency: f64,
  /// The scale of the settled areas i.e. areas in which buildings can be generated: the higher the frequency, the
  /// smaller the areas in which buildings are generated. A parameter of [`noise::BasicMulti<noise::Perlin>`].
  #[inspector(min = 0.0, max = 0.8, display = NumberDisplay::Slider)]
  pub settlement_noise_frequency: f64,
  /// The likelihood of a chunk being considered "settled" and therefore eligible for building generation. Used to
  /// determine the threshold from the settlement noise map above which a chunk is considered settled.
  #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
  pub settlement_probability: f64,
}

impl Default for GenerationMetadataSettings {
  fn default() -> Self {
    Self {
      elevation_chunk_step_size: ELEVATION_CHUNK_STEP_SIZE,
      elevation_offset: ELEVATION_OFFSET,
      biome_noise_frequency: BIOME_NOISE_FREQUENCY,
      settlement_noise_frequency: SETTLEMENT_NOISE_FREQUENCY,
      settlement_probability: SETTLEMENT_PROBABILITY,
    }
  }
}

#[derive(Resource, Reflect, InspectorOptions, Clone, Copy)]
#[reflect(Resource, InspectorOptions)]
pub struct WorldGenerationSettings {
  /// The seed for the noise function. A parameter of [`noise::BasicMulti`]. Allows for the same terrain to be
  /// generated i.e. the same seed will always generate the exact same terrain.
  #[inspector(min = 0, max = 100, display = NumberDisplay::Slider)]
  pub noise_seed: u32,
  /// The overall impact of the noise function on the terrain generation. A simple multiplier for the final output of
  /// the Perlin noise function. The lower the value, the higher the impact of other parameters such as the elevation
  /// offset from the [`crate::generation::model::ElevationMetadata`].
  #[inspector(min = 0., max = 1., display = NumberDisplay::Slider)]
  pub noise_strength: f64,
  /// The amount of detail: The higher the octaves, the more detailed the terrain. A parameter of [`noise::BasicMulti`].
  #[inspector(min = 0, max = 12, display = NumberDisplay::Slider)]
  pub noise_octaves: usize,
  #[inspector(min = 0.0, max = 0.25, display = NumberDisplay::Slider)]
  /// The scale: the higher the frequency, the smaller the terrain features. A parameter of [`noise::BasicMulti`].
  pub noise_frequency: f64,
  /// The abruptness of changes in terrain: The higher the persistence, the rougher the terrain. A parameter
  /// of [`noise::BasicMulti`].
  #[inspector(min = 0., max = 2., display = NumberDisplay::Slider)]
  pub noise_persistence: f64,
  #[inspector(min = 0., max = 10., display = NumberDisplay::Slider)]
  /// The higher the amplitude, the more extreme the terrain. Similar to `noise_persistence` but applies to the entire
  /// output of the noise function equally. A custom parameter that is not part of [`noise::BasicMulti`].
  pub noise_amplitude: f64,
}

impl Default for WorldGenerationSettings {
  fn default() -> Self {
    Self {
      noise_seed: NOISE_SEED,
      noise_strength: NOISE_STRENGTH,
      noise_octaves: NOISE_OCTAVES,
      noise_frequency: NOISE_FREQUENCY,
      noise_persistence: NOISE_PERSISTENCE,
      noise_amplitude: NOISE_AMPLITUDE,
    }
  }
}

#[derive(Resource, Reflect, InspectorOptions, Clone, Copy)]
#[reflect(Resource, InspectorOptions)]
pub struct ObjectGenerationSettings {
  /// Whether to generate objects in the world. If set to `false`, no object grids will be generated, effectively
  /// disabling both path generation and the generation of decorative objects such as trees, stones, flowers, etc.
  pub generate_objects: bool,
  /// Whether to generate paths in the world. Will be ignored if `generate_objects` is `false` because the term objects
  /// captures all non-terrain sprites.
  pub generate_paths: bool,
  /// Whether to generate settlement structures - such as buildings and fields - in the world. Will be ignored if
  /// `generate_paths` is `false` as structures are generated along paths. Will be ignored if `generate_objects` is
  /// `false` because the term objects captures all non-terrain sprites.
  pub generate_settlements: bool,
  /// The density of structures within a settled chunk. The higher the value, the more settlement structures will be
  /// generated within a settled chunk.
  #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
  pub settlement_density: f64,
  /// Whether to generate decorative objects in the world, such as trees, stones, flowers, etc. Will be ignored if
  /// `generate_objects` is `false`.
  pub generate_decoration: bool,
  /// Whether to allow generating objects that have animated sprites. If disabled, only objects with static sprites will
  /// be generated and spawned. This can reduce wave function collapse error rates and improve performance because it
  /// reduces the number of possible object states.
  pub enable_animated_objects: bool,
  /// Whether to enable random colour variations for decorative objects. Does not affect paths or settlement structures.
  pub enable_colour_variations: bool,
}

impl Default for ObjectGenerationSettings {
  fn default() -> Self {
    Self {
      generate_objects: GENERATE_OBJECTS,
      generate_paths: GENERATE_PATHS,
      generate_settlements: GENERATE_SETTLEMENTS,
      settlement_density: SETTLEMENT_DENSITY,
      generate_decoration: GENERATE_DECORATION,
      enable_animated_objects: ENABLE_ANIMATED_OBJECTS,
      enable_colour_variations: ENABLE_COLOUR_VARIATIONS,
    }
  }
}
