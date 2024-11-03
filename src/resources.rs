use crate::constants::*;
use crate::coords::point::{World, WorldGrid};
use crate::coords::Point;
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
      .insert_resource(CurrentChunk::default());
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
  center_world: Point<World>,
  world: Point<World>,
  world_grid: Point<WorldGrid>,
}

#[allow(dead_code)]
impl CurrentChunk {
  pub fn get_world(&self) -> Point<World> {
    self.world
  }

  pub fn get_center_world(&self) -> Point<World> {
    self.center_world
  }

  pub fn get_world_grid(&self) -> Point<WorldGrid> {
    self.world_grid
  }

  pub fn is_world_in_chunk(&self, world: Point<World>) -> bool {
    world.x >= self.world.x
      && world.x < (self.world.x + (CHUNK_SIZE * TILE_SIZE as i32))
      && world.y >= self.world.y
      && world.y < (self.world.y + (CHUNK_SIZE * TILE_SIZE as i32))
  }

  pub fn contains(&self, world_grid: Point<WorldGrid>) -> bool {
    world_grid.x >= self.world_grid.x
      && world_grid.x < (self.world_grid.x + CHUNK_SIZE)
      && world_grid.y >= self.world_grid.y
      && world_grid.y < (self.world_grid.y - CHUNK_SIZE)
  }

  pub fn update(&mut self, world: Point<World>) {
    let old_value = self.world;
    self.world = world;
    self.world_grid = Point::new_world_grid_from_world(world);
    self.center_world = Point::new_world(
      world.x + (CHUNK_SIZE * TILE_SIZE as i32 / 2),
      world.y - (CHUNK_SIZE * TILE_SIZE as i32 / 2),
    );
    debug!("CurrentChunk updated from w{} to w{}", old_value, self.world);
  }
}

impl Default for CurrentChunk {
  fn default() -> Self {
    let world = Point::new_world_from_world_grid(ORIGIN_WORLD_GRID_SPAWN_POINT);
    Self {
      center_world: Point::new_world(
        world.x + (CHUNK_SIZE * TILE_SIZE as i32 / 2),
        world.y - (CHUNK_SIZE * TILE_SIZE as i32 / 2),
      ),
      world,
      world_grid: ORIGIN_WORLD_GRID_SPAWN_POINT,
    }
  }
}
