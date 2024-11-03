use crate::constants::{CHUNK_SIZE, DESPAWN_DISTANCE, ORIGIN_WORLD_SPAWN_POINT, TILE_SIZE};
use crate::coords::point::World;
use crate::coords::Point;
use crate::events::{PruneWorldEvent, RegenerateWorldEvent, UpdateWorldEvent};
use crate::generation::debug::DebugPlugin;
use crate::generation::lib::{
  get_direction_points, ChunkComponent, Direction, UpdateWorldComponent, UpdateWorldStatus, WorldComponent,
};
use crate::generation::object::ObjectGenerationPlugin;
use crate::generation::resources::{ChunkComponentIndex, GenerationResourcesCollection};
use crate::generation::world::WorldGenerationPlugin;
use crate::resources::{CurrentChunk, Settings};
use crate::states::AppState;
use bevy::app::{App, Plugin};
use bevy::core::Name;
use bevy::hierarchy::BuildChildren;
use bevy::log::*;
use bevy::prelude::{
  in_state, Commands, DespawnRecursiveExt, Entity, EventReader, EventWriter, IntoSystemConfigs, Local, NextState, OnEnter,
  Query, Res, ResMut, SpatialBundle, Update, With,
};
use bevy::tasks::{block_on, poll_once, AsyncComputeTaskPool};
use rand::prelude::StdRng;
use rand::SeedableRng;
use resources::GenerationResourcesPlugin;
use std::time::SystemTime;

mod async_utils;
mod debug;
pub(crate) mod lib;
mod object;
pub mod resources;
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
      .add_systems(OnEnter(AppState::Initialising), generation_system)
      .add_systems(
        Update,
        (
          regenerate_world_event,
          update_world_event,
          prune_world_event,
          update_world_system,
        )
          .run_if(in_state(AppState::Running)),
      );
  }
}

/// Generates the world and all its objects. Called once after resources have been loaded.
fn generation_system(
  commands: Commands,
  resources: Res<GenerationResourcesCollection>,
  settings: Res<Settings>,
  mut next_state: ResMut<NextState<AppState>>,
) {
  generate(commands, resources, settings);
  next_state.set(AppState::Running);
  debug!("Transitioning to [{:?}] state", AppState::Running);
}

/// Destroys the world and then generates a new one and all its objects. Called when a `RegenerateWorldEvent` is
/// received. This is triggered by pressing a key or a button in the UI.
fn regenerate_world_event(
  mut commands: Commands,
  mut events: EventReader<RegenerateWorldEvent>,
  existing_world: Query<Entity, With<WorldComponent>>,
  resources: Res<GenerationResourcesCollection>,
  settings: Res<Settings>,
) {
  let event_count = events.read().count();
  if event_count > 0 {
    let world = existing_world.get_single().expect("Failed to get existing world entity");
    commands.entity(world).despawn_recursive();
    generate(commands, resources, settings);
  }
}

/// Generates the world and all its objects. Used by `generation_system` and `regenerate_world_event`.
fn generate(mut commands: Commands, _resources: Res<GenerationResourcesCollection>, settings: Res<Settings>) {
  let start_time = get_time();
  let w = ORIGIN_WORLD_SPAWN_POINT;
  let settings = settings.clone();
  let mut spawn_points: Vec<Point<World>> = vec![w];
  if settings.general.generate_neighbour_chunks {
    let neighbours: Vec<Point<World>> = get_direction_points(&w).iter().map(|(_, chunk)| chunk.clone()).collect();
    spawn_points.extend(neighbours)
  }
  let task_pool = AsyncComputeTaskPool::get();
  let task = task_pool.spawn(async move { world::generate_chunks(spawn_points, &settings) });
  commands.spawn((Name::new("World"), SpatialBundle::default(), WorldComponent));
  commands.spawn((
    Name::new(format!("Update World Component {}", w)),
    UpdateWorldComponent::new(w, task, false, get_time()),
  ));
  info!("Scheduled world generation which took {} ms", get_time() - start_time);
}

/// Updates the world and all its objects. Called when an `UpdateWorldEvent` is received. Triggered when the camera
/// moves outside the `CurrentChunk` or when manually requesting a world re-generation while the camera is within the
/// bounds of the `Chunk` at `ORIGIN_SPAWN_POINT`.
fn update_world_event(
  mut commands: Commands,
  mut events: EventReader<UpdateWorldEvent>,
  existing_chunks: Res<ChunkComponentIndex>,
  mut current_chunk: ResMut<CurrentChunk>,
  settings: Res<Settings>,
) {
  for event in events.read() {
    // Ignore the event if the current chunk contains the world grid of the event
    if current_chunk.contains(event.tg) && !event.is_forced_update {
      debug!("{} is inside current chunk, ignoring event...", event.tg);
      return;
    }

    // Calculate the new current chunk and the chunks to spawn, then spawn UpdateWorldComponent to kick off processing
    let settings = settings.clone();
    let new_parent_w = calculate_new_current_chunk_w(&mut current_chunk, &event);
    let spawn_points = calculate_chunk_spawn_points(&existing_chunks, &settings, &new_parent_w);
    let task_pool = AsyncComputeTaskPool::get();
    let task = task_pool.spawn(async move { world::generate_chunks(spawn_points, &settings) });
    commands.spawn((
      Name::new(format!("Update World Component {}", new_parent_w)),
      UpdateWorldComponent::new(new_parent_w, task, event.is_forced_update, get_time()),
    ));
    current_chunk.update(new_parent_w);
  }
}

fn calculate_new_current_chunk_w(current_chunk: &mut CurrentChunk, event: &UpdateWorldEvent) -> Point<World> {
  let current_chunk_w = current_chunk.get_world();
  let direction = Direction::from_chunk(&current_chunk_w, &event.w);
  let direction_point_w = Point::<World>::from_direction(&direction);
  let new_parent_chunk_w = Point::new_world(
    current_chunk_w.x + (CHUNK_SIZE * TILE_SIZE as i32 * direction_point_w.x),
    current_chunk_w.y + (CHUNK_SIZE * TILE_SIZE as i32 * direction_point_w.y),
  );
  trace!(
    "Update world event at {} {} will change the current chunk to be at [{:?}] of {} i.e. {}",
    event.w,
    event.tg,
    direction,
    current_chunk_w,
    new_parent_chunk_w
  );

  new_parent_chunk_w
}

fn calculate_chunk_spawn_points(
  existing_chunks: &Res<ChunkComponentIndex>,
  settings: &Settings,
  new_parent_chunk_w: &Point<World>,
) -> Vec<Point<World>> {
  let mut spawn_points = Vec::new();
  get_direction_points(&new_parent_chunk_w)
    .iter()
    .for_each(|(direction, chunk_w)| {
      if let Some(_) = existing_chunks.get(*chunk_w) {
        trace!("✅  [{:?}] chunk at {:?} already exists", direction, chunk_w);
      } else {
        if !settings.general.generate_neighbour_chunks && chunk_w != new_parent_chunk_w {
          trace!(
            "❎  [{:?}] chunk at {:?} skipped because generating neighbours is disabled",
            direction,
            chunk_w
          );
          return;
        }
        trace!("🚫 [{:?}] chunk at {:?} needs to be generated", direction, chunk_w);
        spawn_points.push(chunk_w.clone());
      }
    });

  spawn_points
}

/// Updates the world and all its objects. This is the core system that drives the generation of the world and all its
/// objects once a `UpdateWorldComponent` has been spawned.
fn update_world_system(
  mut commands: Commands,
  existing_world: Query<Entity, With<WorldComponent>>,
  mut update_world: Query<(Entity, &mut UpdateWorldComponent), With<UpdateWorldComponent>>,
  settings: Res<Settings>,
  resources: Res<GenerationResourcesCollection>,
  mut prune_world_event: EventWriter<PruneWorldEvent>,
) {
  for (entity, mut component) in update_world.iter_mut() {
    let start_time = get_time();
    let world_entity = existing_world.get_single().expect("Failed to get existing world entity");
    match component.status {
      UpdateWorldStatus::GenerateChunks => {
        // Await the generation of the new chunks
        if let Some(task) = component.stage_1_gen_task.as_mut() {
          if task.is_finished() {
            if let Some(object_data) = block_on(poll_once(task)) {
              component.stage_2_chunks = object_data;
              component.stage_1_gen_task = None;
              component.status = UpdateWorldStatus::SpawnEmptyEntities;
            }
          }
        }
        if component.stage_1_gen_task.is_none() {
          component.status = UpdateWorldStatus::SpawnEmptyEntities;
        }
      }
      UpdateWorldStatus::SpawnEmptyEntities => {
        // Spawn entities for the new chunks and all of its tiles
        if !component.stage_2_chunks.is_empty() {
          let chunk = component.stage_2_chunks.remove(0);
          commands.entity(world_entity).with_children(|parent| {
            let tile_data = world::spawn_chunk(parent, &chunk);
            component.stage_3_spawn_data.push((chunk, tile_data));
          });
        }
        if component.stage_2_chunks.is_empty() {
          component.status = UpdateWorldStatus::ScheduleSpawningTiles;
        }
      }
      UpdateWorldStatus::ScheduleSpawningTiles => {
        // Schedule the spawning of all the tile sprites in the new chunks
        if !component.stage_3_spawn_data.is_empty() {
          let spawn_data = component.stage_3_spawn_data.remove(0);
          world::schedule_tile_spawning_tasks(&mut commands, &settings, &vec![spawn_data.clone()]);
          component.stage_4_spawn_data.push(spawn_data);
        }
        if component.stage_3_spawn_data.is_empty() {
          component.status = UpdateWorldStatus::GenerateObjectData;
        }
      }
      UpdateWorldStatus::GenerateObjectData => {
        // Determine any objects to spawn
        if !component.stage_4_spawn_data.is_empty() {
          let spawn_data = component.stage_4_spawn_data.remove(0);
          let resources = resources.clone();
          let settings = settings.clone();
          let task_pool = AsyncComputeTaskPool::get();
          let task = task_pool.spawn(async move { object::generate_object_data(&resources, &settings, spawn_data) });
          component.stage_5_object_data.push(task);
        }
        if component.stage_4_spawn_data.is_empty() {
          component.status = UpdateWorldStatus::ScheduleSpawningObjects;
        }
      }
      UpdateWorldStatus::ScheduleSpawningObjects => {
        // Schedule the spawning of all the objects in the new chunks
        if !component.stage_5_object_data.is_empty() {
          component.stage_5_object_data.retain_mut(|task| {
            if let Some(object_data) = block_on(poll_once(task)) {
              let mut rng = StdRng::seed_from_u64(settings.world.noise_seed as u64);
              object::schedule_spawning_objects(&mut commands, &mut rng, object_data);
              false
            } else {
              true
            }
          });
        }
        if component.stage_5_object_data.is_empty() {
          component.status = UpdateWorldStatus::ScheduleWorldPruning;
        }
      }
      UpdateWorldStatus::ScheduleWorldPruning => {
        // Clean up the world by despawning distant chunks, if necessary
        if !component.prune_world_after {
          prune_world_event.send(PruneWorldEvent {
            despawn_all_chunks: false,
            update_world_after: false,
          });
        }
        component.status = UpdateWorldStatus::Done;
      }
      UpdateWorldStatus::Done => {
        debug!("✅  {} which took {} ms", *component, get_time() - component.created_at);
        commands.entity(entity).despawn_recursive();
        return;
      }
    }
    trace!("{} which took {} ms", *component, get_time() - start_time);
  }
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
        tg: current_chunk.get_tile_grid(),
        w: current_chunk.get_world(),
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
  for chunk_entity in chunks_to_despawn.iter() {
    if let Some(entity) = commands.get_entity(*chunk_entity) {
      entity.despawn_recursive();
    }
  }
  info!(
    "World pruning (despawn_all_chunks={}, update_world_after={}) took {} ms on [{}]",
    despawn_all_chunks,
    update_world_after,
    get_time() - start_time,
    async_utils::get_thread_info()
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
        "Despawning chunk at {:?} because all chunks have to be despawned",
        chunk_component.coords.world
      );
      chunks_to_despawn.push(entity);
      continue;
    }
    let distance = current_chunk.get_world().distance_to(&chunk_component.coords.world);
    if distance > DESPAWN_DISTANCE {
      trace!(
        "Despawning chunk at {:?} because it's {}px away from current chunk at {:?}",
        chunk_component.coords.world,
        distance as i32,
        current_chunk.get_world()
      );
      chunks_to_despawn.push(entity);
    }
  }

  chunks_to_despawn
}

fn get_time() -> u128 {
  SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()
}
