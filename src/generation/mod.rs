use crate::constants::{CHUNK_SIZE, DESPAWN_DISTANCE, ORIGIN_CHUNK_GRID_SPAWN_POINT, ORIGIN_WORLD_SPAWN_POINT, TILE_SIZE};
use crate::coords::point::World;
use crate::coords::Point;
use crate::events::{PruneWorldEvent, RegenerateWorldEvent, UpdateWorldEvent};
use crate::generation::debug::DebugPlugin;
use crate::generation::lib::{
  get_direction_points, ChunkComponent, Direction, GenerationStage, WorldComponent, WorldGenerationComponent,
};
use crate::generation::object::ObjectGenerationPlugin;
use crate::generation::resources::{ChunkComponentIndex, GenerationResourcesCollection, Metadata};
use crate::generation::world::WorldGenerationPlugin;
use crate::resources::{CurrentChunk, Settings};
use crate::states::{AppState, GenerationState};
use bevy::app::{App, Plugin};
use bevy::core::Name;
use bevy::hierarchy::BuildChildren;
use bevy::log::*;
use bevy::prelude::{
  in_state, Commands, DespawnRecursiveExt, Entity, EventReader, EventWriter, IntoSystemConfigs, Local, Mut, NextState,
  OnExit, OnRemove, Query, Res, ResMut, Transform, Trigger, Update, Visibility, With,
};
use bevy::tasks::{block_on, poll_once, AsyncComputeTaskPool};
use lib::shared;
use rand::prelude::StdRng;
use rand::SeedableRng;
use resources::GenerationResourcesPlugin;

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
      .add_systems(OnExit(AppState::Initialising), initiate_world_generation_system)
      .add_systems(Update, world_generation_system.run_if(in_state(GenerationState::Generating)))
      .add_systems(
        Update,
        (regenerate_world_event, update_world_event, prune_world_event).run_if(in_state(AppState::Running)),
      )
      .add_observer(on_remove_update_world_component_trigger);
  }
}

/// Generates the world and all its objects. Called once before entering `AppState::Running`.
fn initiate_world_generation_system(mut commands: Commands, mut next_state: ResMut<NextState<GenerationState>>) {
  let w = ORIGIN_WORLD_SPAWN_POINT;
  let cg = ORIGIN_CHUNK_GRID_SPAWN_POINT;
  debug!("Generating world with origin {} {}", w, cg);
  commands.spawn((
    Name::new(format!("Update World Component {}", w)),
    WorldGenerationComponent::new(w, cg, false, shared::get_time()),
  ));
  commands.spawn((
    Name::new("World"),
    Transform::default(),
    Visibility::default(),
    WorldComponent,
  ));
  next_state.set(GenerationState::Generating);
}

/// Destroys the world and then generates a new one and all its objects. Called when a `RegenerateWorldEvent` is
/// received. This is triggered by pressing a key or a button in the UI while the camera is within the bounds of the
/// `Chunk` at the origin of the world.
fn regenerate_world_event(
  mut commands: Commands,
  mut events: EventReader<RegenerateWorldEvent>,
  existing_world: Query<Entity, With<WorldComponent>>,
  mut next_state: ResMut<NextState<GenerationState>>,
) {
  let event_count = events.read().count();
  if event_count > 0 {
    let world = existing_world.get_single().expect("Failed to get existing world entity");
    let w = ORIGIN_WORLD_SPAWN_POINT;
    let cg = ORIGIN_CHUNK_GRID_SPAWN_POINT;
    debug!("Regenerating world with origin {} {}", w, cg);
    commands.entity(world).despawn_recursive();
    commands.spawn((
      Name::new(format!("Update World Component {}", cg)),
      WorldGenerationComponent::new(w, cg, false, shared::get_time()),
    ));
    commands.spawn((
      Name::new("World"),
      Transform::default(),
      Visibility::default(),
      WorldComponent,
    ));
    next_state.set(GenerationState::Generating);
  }
}

/// Updates the world and all its objects. Called when an `UpdateWorldEvent` is received. Triggered when the camera
/// moves outside the bounds of the `CurrentChunk` or when manually requesting a world re-generation while the camera
/// is outside the bounds of the `Chunk` at origin spawn point.
fn update_world_event(
  mut commands: Commands,
  mut events: EventReader<UpdateWorldEvent>,
  mut current_chunk: ResMut<CurrentChunk>,
  mut next_state: ResMut<NextState<GenerationState>>,
) {
  for event in events.read() {
    if current_chunk.contains(event.tg) && !event.is_forced_update {
      debug!("{} is inside current chunk, ignoring event...", event.tg);
      return;
    }
    let new_parent_w = calculate_new_current_chunk_w(&mut current_chunk, &event);
    let new_parent_cg = Point::new_chunk_grid_from_world(new_parent_w);
    debug!("Updating world with new current chunk at {} {}", new_parent_w, new_parent_cg);
    commands.spawn((
      Name::new(format!("Update World Component {}", new_parent_w)),
      WorldGenerationComponent::new(new_parent_w, new_parent_cg, event.is_forced_update, shared::get_time()),
    ));
    current_chunk.update(new_parent_w);
    next_state.set(GenerationState::Generating);
  }
}

// TODO: Refactor this and ChunkComponentIndex to use cg instead of w
fn calculate_new_current_chunk_w(current_chunk: &mut CurrentChunk, event: &UpdateWorldEvent) -> Point<World> {
  let current_chunk_w = current_chunk.get_world();
  let direction = Direction::from_chunk_w(&current_chunk_w, &event.w);
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

/// Updates the world and all its objects. This is the core system that drives the generation of the world and all its
/// objects. It is triggered when a `WorldGenerationComponent` is spawned.
fn world_generation_system(
  mut commands: Commands,
  existing_world: Query<Entity, With<WorldComponent>>,
  mut world_generation_components: Query<(Entity, &mut WorldGenerationComponent), With<WorldGenerationComponent>>,
  settings: Res<Settings>,
  metadata: Res<Metadata>,
  resources: Res<GenerationResourcesCollection>,
  existing_chunks: Res<ChunkComponentIndex>,
  mut prune_world_event: EventWriter<PruneWorldEvent>,
) {
  for (entity, mut component) in world_generation_components.iter_mut() {
    let start_time = shared::get_time();
    let world_entity = existing_world.get_single().expect("Failed to get existing world entity");
    match component.stage {
      GenerationStage::Stage1 => stage_1_schedule_chunk_generation(&settings, &metadata, &existing_chunks, &mut component),
      GenerationStage::Stage2 => stage_2_await_chunk_generation(&mut component, &existing_chunks),
      GenerationStage::Stage3 => {
        stage_3_spawn_chunks_and_empty_tiles(&mut commands, &mut component, world_entity, &existing_chunks)
      }
      GenerationStage::Stage4 => stage_4_schedule_spawning_tiles(&mut commands, &settings, &mut component),
      GenerationStage::Stage5 => stage_5_schedule_generating_object_data(&settings, &resources, &mut component),
      GenerationStage::Stage6 => stage_6_schedule_spawning_objects(&mut commands, &settings, &mut component),
      GenerationStage::Stage7 => stage_7_clean_up(&mut commands, &mut prune_world_event, entity, &mut component, &settings),
    }
    trace!(
      "World generation component {} reached stage [{:?}] which took {} ms",
      component.cg,
      component.stage,
      shared::get_time() - start_time
    );
  }
}

fn stage_1_schedule_chunk_generation(
  settings: &Settings,
  metadata: &Metadata,
  existing_chunks: &Res<ChunkComponentIndex>,
  component: &mut Mut<WorldGenerationComponent>,
) {
  if !component.stage_0_metadata {
    if metadata.index.contains(&component.cg) {
      component.stage_0_metadata = true;
    } else {
      debug!("Awaiting metadata for {:?}", component.cg);
    }
  }
  if component.stage_0_metadata {
    let settings = settings.clone();
    let metadata = metadata.clone();
    let spawn_points = calculate_chunk_spawn_points(&existing_chunks, &settings, &component.w);
    let task_pool = AsyncComputeTaskPool::get();
    let task = task_pool.spawn(async move { world::generate_chunks(spawn_points, metadata, &settings) });
    component.stage_1_gen_task = Some(task);
    component.stage = GenerationStage::Stage2;
  }
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
      if let Some(_) = existing_chunks.get(&chunk_w) {
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

fn stage_2_await_chunk_generation(component: &mut Mut<WorldGenerationComponent>, existing_chunks: &ChunkComponentIndex) {
  if let Some(task) = component.stage_1_gen_task.as_mut() {
    if task.is_finished() {
      if let Some(mut chunks) = block_on(poll_once(task)) {
        chunks.retain_mut(|chunk| existing_chunks.get(&chunk.coords.world).is_none());
        component.stage_2_chunks = chunks;
        component.stage_1_gen_task = None;
        component.stage = GenerationStage::Stage3;
      }
    }
  }
  if component.stage_1_gen_task.is_none() {
    component.stage = GenerationStage::Stage3;
  }
}

fn stage_3_spawn_chunks_and_empty_tiles(
  commands: &mut Commands,
  component: &mut Mut<WorldGenerationComponent>,
  world_entity: Entity,
  existing_chunks: &Res<ChunkComponentIndex>,
) {
  if !component.stage_2_chunks.is_empty() {
    let chunk = component.stage_2_chunks.remove(0);
    if existing_chunks.get(&chunk.coords.world).is_none() {
      commands.entity(world_entity).with_children(|parent| {
        let tile_data = world::spawn_chunk(parent, &chunk);
        component.stage_3_spawn_data.push((chunk, tile_data));
      });
    }
  }
  if component.stage_2_chunks.is_empty() {
    component.stage = GenerationStage::Stage4;
  }
}

fn stage_4_schedule_spawning_tiles(
  mut commands: &mut Commands,
  settings: &Res<Settings>,
  component: &mut Mut<WorldGenerationComponent>,
) {
  if !component.stage_3_spawn_data.is_empty() {
    let spawn_data = component.stage_3_spawn_data.remove(0);
    world::schedule_tile_spawning_tasks(&mut commands, &settings, spawn_data.clone());
    component.stage_4_spawn_data.push(spawn_data);
  }
  if component.stage_3_spawn_data.is_empty() {
    component.stage = GenerationStage::Stage5;
  }
}

fn stage_5_schedule_generating_object_data(
  settings: &Settings,
  resources: &GenerationResourcesCollection,
  component: &mut Mut<WorldGenerationComponent>,
) {
  if !component.stage_4_spawn_data.is_empty() {
    let spawn_data = component.stage_4_spawn_data.remove(0);
    let resources = resources.clone();
    let settings = settings.clone();
    let task_pool = AsyncComputeTaskPool::get();
    let task = task_pool.spawn(async move { object::generate_object_data(&resources, &settings, spawn_data) });
    component.stage_5_object_data.push(task);
  }
  if component.stage_4_spawn_data.is_empty() {
    component.stage = GenerationStage::Stage6;
  }
}

fn stage_6_schedule_spawning_objects(
  mut commands: &mut Commands,
  settings: &Settings,
  component: &mut Mut<WorldGenerationComponent>,
) {
  if !component.stage_5_object_data.is_empty() {
    let cg = component.cg;
    component.stage_5_object_data.retain_mut(|task| {
      if task.is_finished() {
        let object_data = block_on(poll_once(task)).expect("Failed to get object data");
        let mut rng = StdRng::seed_from_u64(shared::calculate_seed(cg, settings.world.noise_seed));
        object::schedule_spawning_objects(&mut commands, &settings, &mut rng, object_data);
        false
      } else {
        true
      }
    });
  }
  if component.stage_5_object_data.is_empty() {
    component.stage = GenerationStage::Stage7;
  }
}

fn stage_7_clean_up(
  commands: &mut Commands,
  prune_world_event: &mut EventWriter<PruneWorldEvent>,
  entity: Entity,
  component: &mut Mut<WorldGenerationComponent>,
  settings: &Res<Settings>,
) {
  if !component.suppress_pruning_world && settings.general.enable_world_pruning {
    prune_world_event.send(PruneWorldEvent {
      despawn_all_chunks: false,
      update_world_after: false,
    });
  }
  info!(
    "✅  World generation component {} successfully processed in {} ms",
    component.cg,
    shared::get_time() - component.created_at
  );
  commands.entity(entity).despawn_recursive();
}

/// Sets the `GenerationState` to `Idling` when the last `UpdateWorldComponent` has just been removed.
fn on_remove_update_world_component_trigger(
  _trigger: Trigger<OnRemove, WorldGenerationComponent>,
  query: Query<&WorldGenerationComponent>,
  mut next_state: ResMut<NextState<GenerationState>>,
) {
  if query.iter().len() == 1 {
    next_state.set(GenerationState::Idling);
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
  let start_time = shared::get_time();
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
    shared::get_time() - start_time,
    shared::thread_name()
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
        chunk_component.coords.chunk_grid
      );
      chunks_to_despawn.push(entity);
      continue;
    }
    let distance = current_chunk.get_world().distance_to(&chunk_component.coords.world);
    if distance > DESPAWN_DISTANCE {
      trace!(
        "Despawning chunk at {:?} because it's {}px away from current chunk at {:?}",
        chunk_component.coords.chunk_grid,
        distance as i32,
        current_chunk.get_chunk_grid()
      );
      chunks_to_despawn.push(entity);
    }
  }

  chunks_to_despawn
}
