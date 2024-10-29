use crate::constants::{CHUNK_SIZE, DESPAWN_DISTANCE, TILE_SIZE};
use crate::coords::point::World;
use crate::coords::Point;
use crate::events::{PruneWorldEvent, RegenerateWorldEvent, UpdateWorldEvent};
use crate::generation::async_utils::AsyncTask;
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
use bevy::ecs::world::CommandQueue;
use bevy::hierarchy::BuildChildren;
use bevy::log::*;
use bevy::prelude::{
  Commands, Component, DespawnRecursiveExt, Entity, EventReader, EventWriter, Local, NextState, OnEnter, Query, Res, ResMut,
  Update, With,
};
use bevy::tasks;
use bevy::tasks::{block_on, AsyncComputeTaskPool, Task};
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
          handle_update_world_task,
          update_world_system,
        ),
      );
  }
}

#[derive(Component)]
struct UpdateWorldTask(Task<CommandQueue>);

impl AsyncTask for UpdateWorldTask {
  fn poll_once(&mut self) -> Option<CommandQueue> {
    block_on(tasks::poll_once(&mut self.0))
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
fn generate(mut commands: Commands, resources: Res<GenerationResourcesCollection>, settings: Res<Settings>) {
  let start_time = get_time();
  let spawn_data = world::generate_world(&mut commands, &settings);
  object::generate(spawn_data, &resources, &settings, &mut commands);
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
  existing_chunks: Res<ChunkComponentIndex>,
  mut current_chunk: ResMut<CurrentChunk>,
  settings: Res<Settings>,
) {
  for event in events.read() {
    // Ignore the event if the current chunk contains the world grid of the event
    if current_chunk.contains(event.world_grid) && !event.is_forced_update {
      debug!("wg{} is inside current chunk, ignoring UpdateWorldEvent...", event.world_grid);
      return;
    }

    // Calculate the new current chunk and the chunks to spawn, then spawn UpdateWorldComponent to kick off processing
    let start_time = get_time();
    let is_forced_update = event.is_forced_update;
    let task_pool = AsyncComputeTaskPool::get();
    let new_parent_cg = calculate_new_current_cg(&mut current_chunk, &event);
    let chunks_to_spawn = calculate_new_chunks_to_spawn(&existing_chunks, &settings, &new_parent_cg);
    let task = task_pool.spawn(async move {
      let mut command_queue = CommandQueue::default();
      command_queue.push(move |world: &mut bevy::prelude::World| {
        let (_resources, settings) = async_utils::get_resources_and_settings(world);
        let chunks = world::calculate_chunks(chunks_to_spawn, &settings);
        world.spawn((
          Name::new(format!("Update World Component for wg{} Parent", new_parent_cg)),
          UpdateWorldComponent {
            status: UpdateWorldStatus::SpawnEmptyEntities,
            stage_1_chunks: chunks,
            stage_2_spawn_data: Vec::new(),
            stage_3_spawn_data: Vec::new(),
            force_update_after: is_forced_update,
          },
        ));
      });
      command_queue
    });
    commands.spawn((Name::new("Update World Task"), UpdateWorldTask(task)));
    current_chunk.update(new_parent_cg);
    info!("Scheduling world update took {} ms", get_time() - start_time);
  }
}

fn handle_update_world_task(commands: Commands, query: Query<(Entity, &mut UpdateWorldTask)>) {
  async_utils::process_tasks(commands, query);
}

fn calculate_new_current_cg(current_chunk: &mut CurrentChunk, event: &UpdateWorldEvent) -> Point<World> {
  let current_chunk_world = current_chunk.get_world();
  let direction = Direction::from_chunk(&current_chunk_world, &event.world);
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
  settings: &Settings,
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

fn update_world_system(
  mut commands: Commands,
  existing_world: Query<Entity, With<WorldComponent>>,
  mut update_world: Query<(Entity, &mut UpdateWorldComponent), With<UpdateWorldComponent>>,
  settings: Res<Settings>,
  resources: Res<GenerationResourcesCollection>,
  mut clean_up_event: EventWriter<PruneWorldEvent>,
) {
  for (entity, mut component) in update_world.iter_mut() {
    let start_time = get_time();
    let world_entity = existing_world.get_single().expect("Failed to get existing world entity");
    match component.status {
      UpdateWorldStatus::SpawnEmptyEntities => {
        if !component.stage_1_chunks.is_empty() {
          let chunk = component.stage_1_chunks.remove(0);
          commands.entity(world_entity).with_children(|parent| {
            let tile_data = world::spawn_chunk(parent, &chunk);
            component.stage_2_spawn_data.push((chunk, tile_data));
          });
        } else {
          component.status = UpdateWorldStatus::ScheduleTileSpawning;
        }
      }
      UpdateWorldStatus::ScheduleTileSpawning => {
        if !component.stage_2_spawn_data.is_empty() {
          let spawn_data = component.stage_2_spawn_data.remove(0);
          world::schedule_tile_spawning_tasks(&mut commands, &settings, &vec![spawn_data.clone()]);
          component.stage_3_spawn_data.push(spawn_data);
        } else {
          component.status = UpdateWorldStatus::GenerateObjects;
        }
      }
      UpdateWorldStatus::GenerateObjects => {
        if !component.stage_3_spawn_data.is_empty() {
          let spawn_data = component.stage_3_spawn_data.remove(0);
          object::generate(vec![spawn_data], &resources, &settings, &mut commands);
        } else {
          component.status = UpdateWorldStatus::ScheduleWorldPruning;
        }
      }
      UpdateWorldStatus::ScheduleWorldPruning => {
        // // Update the current chunk and clean up the world, if necessary
        if !component.force_update_after {
          clean_up_event.send(PruneWorldEvent {
            despawn_all_chunks: false,
            update_world_after: false,
          });
        }
        component.status = UpdateWorldStatus::Done;
      }
      UpdateWorldStatus::Done => {
        commands.entity(entity).despawn_recursive();
      }
    }
    debug!("Processing {} took {} ms", *component, get_time() - start_time);
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
