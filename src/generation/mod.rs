use crate::constants::{CHUNK_SIZE, TILE_SIZE};
use crate::coords::Point;
use crate::events::{DespawnDistantChunkEvent, RegenerateWorldEvent, UpdateWorldEvent};
use crate::generation::components::{ChunkComponent, WorldComponent};
use crate::generation::debug::DebugPlugin;
use crate::generation::direction::get_direction_points;
use crate::generation::object::ObjectGenerationPlugin;
use crate::generation::resources::{AssetPacks, ChunkComponentIndex, GenerationResourcesPlugin};
use crate::generation::world::WorldGenerationPlugin;
use crate::resources::{CurrentChunk, Settings};
use bevy::app::{App, Plugin};
use bevy::prelude::*;
use std::time::SystemTime;

mod chunk;
mod components;
mod debug;
pub(crate) mod direction;
mod draft_chunk;
mod layered_plane;
mod neighbours;
mod object;
mod plane;
mod resources;
mod terrain_type;
mod tile;
mod tile_data;
mod tile_type;
mod world;

pub struct GenerationPlugin;

impl Plugin for GenerationPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_plugins((
        GenerationResourcesPlugin,
        WorldGenerationPlugin,
        ObjectGenerationPlugin,
        DebugPlugin,
      ))
      .add_systems(Startup, generation_system)
      .add_systems(
        Update,
        (regenerate_world_event, update_world_event, despawn_distant_chunks_event),
      );
  }
}

fn generation_system(commands: Commands, asset_packs: Res<AssetPacks>, settings: Res<Settings>) {
  generate(commands, asset_packs, settings);
}

fn regenerate_world_event(
  mut commands: Commands,
  mut events: EventReader<RegenerateWorldEvent>,
  existing_world: Query<Entity, With<WorldComponent>>,
  asset_packs: Res<AssetPacks>,
  settings: Res<Settings>,
) {
  let event_count = events.read().count();
  if event_count > 0 {
    let world = existing_world.get_single().unwrap();
    commands.entity(world).despawn_recursive();
    generate(commands, asset_packs, settings);
  }
}

fn generate(mut commands: Commands, asset_packs: Res<AssetPacks>, settings: Res<Settings>) {
  let start_time = get_time();
  let mut spawn_data = world::spawn_world(&mut commands, &settings);
  object::generate_objects(&mut commands, &mut spawn_data, &asset_packs, &settings);
  info!("✅  World generation took {} ms", get_time() - start_time);
}

fn get_time() -> u128 {
  SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()
}

fn update_world_event(
  mut commands: Commands,
  mut events: EventReader<UpdateWorldEvent>,
  existing_world: Query<Entity, With<WorldComponent>>,
  existing_chunks: Res<ChunkComponentIndex>,
  mut current_chunk: ResMut<CurrentChunk>,
  asset_packs: Res<AssetPacks>,
  settings: Res<Settings>,
  mut despawn_distant_chunk_event: EventWriter<DespawnDistantChunkEvent>,
) {
  for event in events.read() {
    let start_time = get_time();
    let event_world_grid = event.world_grid;
    let event_world = event.world;
    if current_chunk.contains(event_world_grid) {
      debug!("wg{} is inside current chunk, ignoring UpdateWorldEvent...", event_world_grid);
      return;
    }

    let current_chunk_world = current_chunk.get_world();
    let direction = direction::Direction::from_chunk(&current_chunk_world, &event_world);
    let new_parent_chunk_world = calculate_new_current_chunk_world(&current_chunk_world, &direction);
    debug!(
      "Update world event at w{} wg{}: new current chunk will be at [{:?}] of w{} i.e. w{}",
      event_world, event_world_grid, direction, current_chunk_world, new_parent_chunk_world
    );
    let mut chunks_to_spawn = Vec::new();
    get_direction_points(&new_parent_chunk_world)
      .iter()
      .for_each(|(direction, chunk_world)| {
        if let Some(_) = existing_chunks.get(*chunk_world) {
          trace!("✅  {:?} chunk at w{:?} already exists", direction, chunk_world);
        } else {
          if !settings.general.generate_neighbour_chunks && chunk_world != &new_parent_chunk_world {
            trace!(
              "❎  {:?} chunk at w{:?} skipped because generating neighbours is disabled",
              direction,
              chunk_world
            );
            return;
          }
          debug!("🚫 {:?} chunk at w{:?} needs to be generated", direction, chunk_world);
          chunks_to_spawn.push(chunk_world.clone());
        }
      });

    let world = existing_world.get_single().unwrap();
    let mut spawn_data = world::generate_chunks(&mut commands, world, chunks_to_spawn, &settings);
    object::generate_objects(&mut commands, &mut spawn_data, &asset_packs, &settings);
    world::schedule_tile_spawning_tasks(&mut commands, &settings, &spawn_data);

    current_chunk.update(new_parent_chunk_world);
    despawn_distant_chunk_event.send(DespawnDistantChunkEvent {});
    info!("World update took {} ms", get_time() - start_time);
  }
}

pub fn calculate_new_current_chunk_world(current_chunk_world: &Point, direction: &direction::Direction) -> Point {
  let direction = Point::from_direction(direction, current_chunk_world.coord_type);

  Point::new_world(
    current_chunk_world.x + (CHUNK_SIZE * TILE_SIZE as i32 * direction.x),
    current_chunk_world.y + (CHUNK_SIZE * TILE_SIZE as i32 * direction.y),
  )
}

pub fn despawn_distant_chunks_event(
  mut commands: Commands,
  mut events: EventReader<DespawnDistantChunkEvent>,
  existing_chunks: Query<(Entity, &ChunkComponent), With<ChunkComponent>>,
  current_chunk: Res<CurrentChunk>,
) {
  for _ in events.read() {
    let start_time = get_time();
    let mut chunks_to_despawn = Vec::new();
    for (entity, chunk_component) in existing_chunks.iter() {
      let distance = current_chunk.get_world().distance_to(&chunk_component.coords.world);
      if distance > CHUNK_SIZE as f32 * TILE_SIZE as f32 * 2.0 {
        trace!(
          "Despawning chunk at w{:?} because it's {}px away from current chunk at w{:?}",
          chunk_component.coords.world,
          distance as i32,
          current_chunk.get_world()
        );
        chunks_to_despawn.push(entity);
      }
    }
    for entity in chunks_to_despawn.iter() {
      commands.entity(*entity).despawn_recursive();
    }
    info!("Distant chunks clean up took {} ms", get_time() - start_time);
  }
}
