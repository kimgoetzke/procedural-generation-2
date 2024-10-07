use crate::constants::{CHUNK_SIZE, DESPAWN_DISTANCE, TILE_SIZE};
use crate::coords::point::World;
use crate::coords::Point;
use crate::events::{PruneWorldEvent, RegenerateWorldEvent, UpdateWorldEvent};
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
mod draft_tile;
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
      .add_systems(Update, (regenerate_world_event, update_world_event, prune_world_event));
  }
}

/// Generates the world and all its objects. Called once at startup.
fn generation_system(commands: Commands, asset_packs: Res<AssetPacks>, settings: Res<Settings>) {
  generate(commands, asset_packs, settings);
}

/// Destroys the world and then generates a new one and all its objects. Called when a `RegenerateWorldEvent` is
/// received. This is triggered by pressing a key or a button in the UI.
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

/// Generates the world and all its objects. Used by `generation_system` and `regenerate_world_event`.
fn generate(mut commands: Commands, asset_packs: Res<AssetPacks>, settings: Res<Settings>) {
  let start_time = get_time();
  let mut spawn_data = world::generate_world(&mut commands, &settings);
  object::generate(&mut commands, &mut spawn_data, &asset_packs, &settings);
  info!("✅  World generation took {} ms", get_time() - start_time);
}

fn get_time() -> u128 {
  SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()
}

/// Updates the world and all its objects. Called when an `UpdateWorldEvent` is received. Triggered when the camera
/// moves outside the `CurrentChunk` or manually requesting a world re-generation while the camera is not at the
/// `ORIGIN_SPAWN_POINT`.
fn update_world_event(
  mut commands: Commands,
  mut events: EventReader<UpdateWorldEvent>,
  existing_world: Query<Entity, With<WorldComponent>>,
  existing_chunks: Res<ChunkComponentIndex>,
  mut current_chunk: ResMut<CurrentChunk>,
  asset_packs: Res<AssetPacks>,
  settings: Res<Settings>,
  mut clean_up_event: EventWriter<PruneWorldEvent>,
) {
  for event in events.read() {
    // Ignore the event if the current chunk contains the world grid of the event
    if current_chunk.contains(event.world_grid) && !event.is_forced_update {
      debug!("wg{} is inside current chunk, ignoring UpdateWorldEvent...", event.world_grid);
      return;
    }

    // Generate new chunks and objects
    let start_time = get_time();
    let new_parent_chunk_world = calculate_new_current_chunk_world(&mut current_chunk, &event);
    let chunks_to_spawn = calculate_new_chunks_to_spawn(&existing_chunks, &settings, &new_parent_chunk_world);
    let world = existing_world.get_single().unwrap();
    let mut spawn_data = world::generate_chunks(&mut commands, world, chunks_to_spawn, &settings);
    object::generate(&mut commands, &mut spawn_data, &asset_packs, &settings);

    // Update the current chunk and clean up the world, if necessary
    current_chunk.update(new_parent_chunk_world);
    if !event.is_forced_update {
      clean_up_event.send(PruneWorldEvent {
        despawn_all_chunks: false,
        update_world_after: false,
      });
    }
    info!("World update took {} ms", get_time() - start_time);
  }
}

fn calculate_new_current_chunk_world(current_chunk: &mut ResMut<CurrentChunk>, event: &UpdateWorldEvent) -> Point<World> {
  let current_chunk_world = current_chunk.get_world();
  let direction = direction::Direction::from_chunk(&current_chunk_world, &event.world);
  let direction_point = Point::<World>::from_direction(&direction);
  let new_parent_chunk_world = Point::new_world(
    current_chunk_world.x + (CHUNK_SIZE * TILE_SIZE as i32 * direction_point.x),
    current_chunk_world.y + (CHUNK_SIZE * TILE_SIZE as i32 * direction_point.y),
  );
  trace!(
    "Update world event at w{} wg{} will change the current chunk to be at [{:?}] of w{} i.e. w{}",
    event.world,
    event.world_grid,
    direction,
    current_chunk_world,
    new_parent_chunk_world
  );

  new_parent_chunk_world
}

fn calculate_new_chunks_to_spawn(
  existing_chunks: &Res<ChunkComponentIndex>,
  settings: &Res<Settings>,
  new_parent_chunk_world: &Point<World>,
) -> Vec<Point<World>> {
  let mut chunks_to_spawn = Vec::new();
  get_direction_points(&new_parent_chunk_world)
    .iter()
    .for_each(|(direction, chunk_world)| {
      if let Some(_) = existing_chunks.get(*chunk_world) {
        trace!("✅  [{:?}] chunk at w{:?} already exists", direction, chunk_world);
      } else {
        if !settings.general.generate_neighbour_chunks && chunk_world != new_parent_chunk_world {
          trace!(
            "❎  [{:?}] chunk at w{:?} skipped because generating neighbours is disabled",
            direction,
            chunk_world
          );
          return;
        }
        trace!("🚫 [{:?}] chunk at w{:?} needs to be generated", direction, chunk_world);
        chunks_to_spawn.push(chunk_world.clone());
      }
    });

  chunks_to_spawn
}

pub fn prune_world_event(
  mut commands: Commands,
  mut prune_world_event: EventReader<PruneWorldEvent>,
  mut update_world_event: EventWriter<UpdateWorldEvent>,
  existing_chunks: Query<(Entity, &ChunkComponent), With<ChunkComponent>>,
  current_chunk: Res<CurrentChunk>,
  mut delayed_update_world_event: Local<Option<UpdateWorldEvent>>,
) {
  // Allows the `PruneWorldEvent` to trigger an `UpdateWorldEvent` after the world has been pruned. Doing this in the
  // same frame will lead to race conditions and chunks been despawned just after they were spawned.
  if let Some(event) = delayed_update_world_event.take() {
    update_world_event.send(event);
  }

  for event in prune_world_event.read() {
    prune_world(
      &mut commands,
      &existing_chunks,
      &current_chunk,
      event.despawn_all_chunks,
      event.update_world_after,
    );
    if event.update_world_after {
      *delayed_update_world_event = Some(UpdateWorldEvent {
        is_forced_update: true,
        world_grid: current_chunk.get_world_grid(),
        world: current_chunk.get_world(),
      });
    }
  }
}

fn prune_world(
  commands: &mut Commands,
  existing_chunks: &Query<(Entity, &ChunkComponent), With<ChunkComponent>>,
  current_chunk: &Res<CurrentChunk>,
  despawn_all_chunks: bool,
  update_world_after: bool,
) {
  let start_time = get_time();
  let chunks_to_despawn = calculate_chunks_to_despawn(existing_chunks, current_chunk, despawn_all_chunks);
  for entity in chunks_to_despawn.iter() {
    commands.entity(*entity).despawn_recursive();
  }
  info!(
    "World pruning (despawn_all_chunks={}, update_world_after={}) took {} ms",
    despawn_all_chunks,
    update_world_after,
    get_time() - start_time
  );
}

fn calculate_chunks_to_despawn(
  existing_chunks: &Query<(Entity, &ChunkComponent), With<ChunkComponent>>,
  current_chunk: &Res<CurrentChunk>,
  despawn_all_chunks: bool,
) -> Vec<Entity> {
  let mut chunks_to_despawn = Vec::new();
  for (entity, chunk_component) in existing_chunks.iter() {
    if despawn_all_chunks {
      trace!(
        "Despawning chunk at w{:?} because all chunks have to be despawned",
        chunk_component.coords.world
      );
      chunks_to_despawn.push(entity);
      continue;
    }
    let distance = current_chunk.get_world().distance_to(&chunk_component.coords.world);
    if distance > DESPAWN_DISTANCE {
      trace!(
        "Despawning chunk at w{:?} because it's {}px away from current chunk at w{:?}",
        chunk_component.coords.world,
        distance as i32,
        current_chunk.get_world()
      );
      chunks_to_despawn.push(entity);
    }
  }

  chunks_to_despawn
}
