use crate::constants::*;
use crate::coords::point::{ChunkGrid, TileGrid, World};
use crate::coords::{Coords, Point};
use bevy::app::{App, Plugin};
use bevy::log::*;
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
      .insert_resource(WorldGenerationSettings::default())
      .init_resource::<GenerationMetadataSettings>()
      .register_type::<GenerationMetadataSettings>()
      .insert_resource(GenerationMetadataSettings::default())
      .insert_resource(CurrentChunk::default());
  }
}

#[derive(Resource, Reflect, Clone, Copy)]
pub struct Settings {
  pub general: GeneralGenerationSettings,
  pub metadata: GenerationMetadataSettings,
  pub world: WorldGenerationSettings,
  pub object: ObjectGenerationSettings,
}

impl Default for Settings {
  fn default() -> Self {
    Self {
      general: GeneralGenerationSettings::default(),
      metadata: GenerationMetadataSettings::default(),
      world: WorldGenerationSettings::default(),
      object: ObjectGenerationSettings::default(),
    }
  }
}

#[derive(Resource, Reflect, InspectorOptions, Clone, Copy)]
#[reflect(Resource, InspectorOptions)]
pub struct GeneralGenerationSettings {
  pub draw_gizmos: bool,
  pub generate_neighbour_chunks: bool,
  pub enable_tile_debugging: bool,
  pub draw_terrain_sprites: bool,
  pub animate_terrain_sprites: bool,
  #[inspector(min = 0, max = 4, display = NumberDisplay::Slider)]
  pub spawn_from_layer: usize,
  #[inspector(min = 0, max = 4, display = NumberDisplay::Slider)]
  pub spawn_up_to_layer: usize,
}

impl Default for GeneralGenerationSettings {
  fn default() -> Self {
    Self {
      draw_gizmos: DRAW_GIZMOS,
      generate_neighbour_chunks: GENERATE_NEIGHBOUR_CHUNKS,
      enable_tile_debugging: ENABLE_TILE_DEBUGGING,
      draw_terrain_sprites: DRAW_TERRAIN_SPRITES,
      animate_terrain_sprites: ANIMATE_TERRAIN_SPRITES,
      spawn_from_layer: SPAWN_FROM_LAYER,
      spawn_up_to_layer: SPAWN_UP_TO_LAYER,
    }
  }
}

#[derive(Resource, Reflect, InspectorOptions, Clone, Copy)]
#[reflect(Resource, InspectorOptions)]
pub struct GenerationMetadataSettings {
  /// The increase in elevation for a given chunk's x-axis. The higher the value, faster the terrain goes from
  /// the lowest terrain layer to the highest terrain layer from left to right. Negative values will reverse the
  /// direction.
  #[inspector(min = -0.5, max = 0.5, display = NumberDisplay::Slider)]
  pub elevation_step_increase_x: f32,
  /// The increase in elevation for a given chunk's y-axis. The higher the value, faster the terrain goes from
  /// the lowest terrain layer to the highest terrain layer from top to bottom. Negative values will reverse the
  /// direction.
  #[inspector(min = -0.5, max = 0.5, display = NumberDisplay::Slider)]
  pub elevation_step_increase_y: f32,
}

impl Default for GenerationMetadataSettings {
  fn default() -> Self {
    Self {
      elevation_step_increase_x: ELEVATION_STEP_INCREASE_X,
      elevation_step_increase_y: ELEVATION_STEP_INCREASE_Y,
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
  pub generate_objects: bool,
}

impl Default for ObjectGenerationSettings {
  fn default() -> Self {
    Self {
      generate_objects: GENERATE_OBJECTS,
    }
  }
}

#[derive(Resource, Debug, Clone)]
pub struct CurrentChunk {
  center_w: Point<World>,
  coords: Coords,
}

impl CurrentChunk {
  pub fn get_center_world(&self) -> Point<World> {
    self.center_w
  }

  pub fn get_world(&self) -> Point<World> {
    self.coords.world
  }

  pub fn get_tile_grid(&self) -> Point<TileGrid> {
    self.coords.tile_grid
  }

  pub fn get_chunk_grid(&self) -> Point<ChunkGrid> {
    self.coords.chunk_grid
  }

  pub fn contains(&self, tg: Point<TileGrid>) -> bool {
    tg.x >= self.coords.tile_grid.x
      && tg.x < (self.coords.tile_grid.x + CHUNK_SIZE)
      && tg.y >= self.coords.tile_grid.y
      && tg.y < (self.coords.tile_grid.y - CHUNK_SIZE)
  }

  pub fn update(&mut self, w: Point<World>) {
    let old_value = self.coords.chunk_grid;
    let cg = Point::new_chunk_grid_from_world(w);
    self.coords.world = w;
    self.coords.chunk_grid = cg;
    self.coords.tile_grid = Point::new_tile_grid_from_world(w);
    self.center_w = Point::new_world(
      w.x + (CHUNK_SIZE * TILE_SIZE as i32 / 2),
      w.y - (CHUNK_SIZE * TILE_SIZE as i32 / 2),
    );
    debug!("CurrentChunk updated from {} to {}", old_value, cg);
  }
}

impl Default for CurrentChunk {
  fn default() -> Self {
    Self {
      center_w: Point::new_world(
        ORIGIN_WORLD_SPAWN_POINT.x + (CHUNK_SIZE * TILE_SIZE as i32 / 2),
        ORIGIN_WORLD_SPAWN_POINT.y - (CHUNK_SIZE * TILE_SIZE as i32 / 2),
      ),
      coords: Coords::new(
        ORIGIN_WORLD_SPAWN_POINT,
        ORIGIN_CHUNK_GRID_SPAWN_POINT,
        ORIGIN_TILE_GRID_SPAWN_POINT,
      ),
    }
  }
}
