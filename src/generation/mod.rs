use self::object::path;
use crate::constants::{
  CHUNK_SIZE, DESPAWN_DISTANCE, MAX_CHUNKS, ORIGIN_CHUNK_GRID_SPAWN_POINT, ORIGIN_WORLD_SPAWN_POINT, TILE_SIZE,
};
use crate::coords::Point;
use crate::coords::point::{ChunkGrid, World};
use crate::events::{PruneWorldEvent, RegenerateWorldEvent, UpdateWorldEvent};
use crate::generation::debug::DebugPlugin;
use crate::generation::lib::{
  Chunk, ChunkComponent, Direction, GenerationResourcesCollection, GenerationStage, WorldComponent,
  WorldGenerationComponent, get_direction_points,
};
use crate::generation::object::ObjectGenerationPlugin;
use crate::generation::object::lib::{ObjectData, ObjectGrid};
use crate::generation::resources::{ChunkComponentIndex, Metadata};
use crate::generation::world::WorldGenerationPlugin;
use crate::resources::{CurrentChunk, Settings};
use crate::states::{AppState, GenerationState};
use bevy::app::{App, Plugin};
use bevy::asset::Assets;
use bevy::log::*;
use bevy::prelude::{
  ColorMaterial, Commands, Entity, EventReader, EventWriter, IntoScheduleConfigs, IntoSystem, Local, Mesh, Mut, Name,
  NextState, Observer, OnExit, OnRemove, Query, Res, ResMut, Transform, Trigger, Update, Visibility, With, in_state,
};
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, poll_once};
use lib::shared;
use rand::SeedableRng;
use rand::prelude::StdRng;
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
        (
          regenerate_world_event,
          update_world_event,
          prune_world_event.after(world_generation_system),
        )
          .run_if(in_state(AppState::Running)),
      )
      .world_mut()
      .spawn((
        Observer::new(IntoSystem::into_system(on_remove_world_generation_component_trigger)),
        Name::new("Observer: Remove WorldGenerationComponent"),
      ));
  }
}

/// Generates the world and all its objects. Called once before entering `AppState::Running`.
fn initiate_world_generation_system(mut commands: Commands, mut next_state: ResMut<NextState<GenerationState>>) {
  let w = ORIGIN_WORLD_SPAWN_POINT;
  let cg = ORIGIN_CHUNK_GRID_SPAWN_POINT;
  debug!("Generating world with origin {} {}", w, cg);
  commands.spawn((
    Name::new(format!("World Generation Component {}", w)),
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
    let world = existing_world.single().expect("Failed to get existing world entity");
    let w = ORIGIN_WORLD_SPAWN_POINT;
    let cg = ORIGIN_CHUNK_GRID_SPAWN_POINT;
    debug!("Regenerating world with origin {} {}", w, cg);
    commands.entity(world).despawn();
    commands.spawn((
      Name::new(format!("World Generation Component {}", cg)),
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
      Name::new(format!("World Generation Component {}", new_parent_w)),
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
    event.w, event.tg, direction, current_chunk_w, new_parent_chunk_w
  );

  new_parent_chunk_w
}

/// The system that actually orchestrates the modification of the world and all its objects. This is the core system
/// that drives the generation of the world and all its objects. It is triggered by spawning a
/// [`WorldGenerationComponent`].
fn world_generation_system(
  mut commands: Commands,
  existing_world: Query<Entity, With<WorldComponent>>,
  mut world_generation_components: Query<(Entity, &mut WorldGenerationComponent), With<WorldGenerationComponent>>,
  settings: Res<Settings>,
  metadata: Res<Metadata>,
  resources: Res<GenerationResourcesCollection>,
  existing_chunks: Res<ChunkComponentIndex>,
  mut prune_world_event: EventWriter<PruneWorldEvent>,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<ColorMaterial>>,
) {
  for (entity, mut component) in world_generation_components.iter_mut() {
    let start_time = shared::get_time();
    let world_entity = existing_world.single().expect("Failed to get existing world entity");
    let current_stage = std::mem::replace(&mut component.stage, GenerationStage::Stage9);
    let component_cg = &component.cg;
    component.stage = match current_stage {
      GenerationStage::Stage1(has_metadata) => stage_1_prune_world_and_schedule_chunk_generation(
        &settings,
        &metadata,
        &existing_chunks,
        &component,
        has_metadata,
        &mut prune_world_event,
      ),
      GenerationStage::Stage2(chunk_generation_task) => {
        stage_2_await_chunk_generation_task_completion(&existing_chunks, chunk_generation_task, component_cg)
      }
      GenerationStage::Stage3(chunks) => {
        stage_3_spawn_chunks(&mut commands, world_entity, &existing_chunks, chunks, component_cg)
      }
      GenerationStage::Stage4(chunk_entity_pairs) => stage_4_spawn_tile_meshes(
        &mut commands,
        &settings,
        &resources,
        chunk_entity_pairs,
        &mut meshes,
        &mut materials,
        component_cg,
      ),
      GenerationStage::Stage5(chunk_entity_pairs) => {
        stage_5_schedule_object_grid_generation(&mut commands, &settings, &resources, chunk_entity_pairs, component_cg)
      }
      GenerationStage::Stage6(grid_generation_task) => {
        stage_6_schedule_path_generation(&mut commands, &metadata, grid_generation_task, component_cg)
      }
      GenerationStage::Stage7(path_generation_task) => {
        stage_7_schedule_generating_object_data(&mut commands, &settings, path_generation_task, component_cg)
      }
      GenerationStage::Stage8(object_generation_tasks) => {
        stage_8_schedule_spawning_objects(&mut commands, &settings, object_generation_tasks, component_cg)
      }
      GenerationStage::Stage9 => stage_9_clean_up(
        &mut commands,
        &mut component,
        &settings,
        entity,
        &existing_chunks,
        &mut prune_world_event,
      ),
      GenerationStage::Done => GenerationStage::Done,
    };
    trace!(
      "World generation component {} ({}) reached stage [{}] which took {} ms",
      component.cg,
      entity,
      component.stage,
      shared::get_time() - start_time
    );
  }
}

fn stage_1_prune_world_and_schedule_chunk_generation(
  settings: &Settings,
  metadata: &Metadata,
  existing_chunks: &Res<ChunkComponentIndex>,
  component: &WorldGenerationComponent,
  mut has_metadata: bool,
  prune_event: &mut EventWriter<PruneWorldEvent>,
) -> GenerationStage {
  if !has_metadata && metadata.index.contains(&component.cg) {
    has_metadata = true;
  } else {
    trace!("World generation component {} - Stage 1 | Awaiting metadata...", component.cg);
  }
  if has_metadata {
    if !component.suppress_pruning_world && settings.general.enable_world_pruning {
      prune_event.write(PruneWorldEvent {
        despawn_all_chunks: false,
        update_world_after: false,
      });
    }

    let settings = settings.clone();
    let metadata = metadata.clone();
    let spawn_points = calculate_chunk_spawn_points(&existing_chunks, &settings, &component.w);
    let task_pool = AsyncComputeTaskPool::get();
    let task = task_pool.spawn(async move { world::generate_chunks(spawn_points, metadata, &settings) });
    return GenerationStage::Stage2(task);
  }

  GenerationStage::Stage1(has_metadata)
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
            direction, chunk_w
          );
          return;
        }
        trace!("🚫 [{:?}] chunk at {:?} needs to be generated", direction, chunk_w);
        spawn_points.push(chunk_w.clone());
      }
    });

  spawn_points
}

fn stage_2_await_chunk_generation_task_completion(
  existing_chunks: &ChunkComponentIndex,
  chunk_generation_task: Task<Vec<Chunk>>,
  cg: &Point<ChunkGrid>,
) -> GenerationStage {
  if chunk_generation_task.is_finished() {
    return if let Some(mut chunks) = block_on(poll_once(chunk_generation_task)) {
      chunks.retain_mut(|chunk| existing_chunks.get(&chunk.coords.world).is_none());
      trace!(
        "World generation component {cg} - Stage 2 | {} new chunks need to be spawned",
        chunks.len()
      );

      GenerationStage::Stage3(chunks)
    } else {
      trace!(
        "World generation component {cg} - Stage 2 | Chunk generation task did not return any chunks - they probably exist already..."
      );

      GenerationStage::Stage9
    };
  }

  GenerationStage::Stage2(chunk_generation_task)
}

fn stage_3_spawn_chunks(
  commands: &mut Commands,
  world_entity: Entity,
  existing_chunks: &Res<ChunkComponentIndex>,
  mut chunks: Vec<Chunk>,
  cg: &Point<ChunkGrid>,
) -> GenerationStage {
  if !chunks.is_empty() {
    let mut chunk_entity_pairs = Vec::new();
    for chunk in chunks.drain(0..) {
      if existing_chunks.get(&chunk.coords.world).is_none() {
        commands.entity(world_entity).with_children(|parent| {
          let chunk_entity = world::spawn_chunk(parent, &chunk);
          chunk_entity_pairs.push((chunk, chunk_entity));
        });
      }
    }
    trace!(
      "World generation component {cg} - Stage 3 | {} new chunk(s) were spawned",
      chunk_entity_pairs.len(),
    );
    return GenerationStage::Stage4(chunk_entity_pairs);
  }

  trace!(
    "World generation component {cg} - Stage 3 | Chunk data was empty - assuming world generation component is redundant..."
  );
  GenerationStage::Stage9
}

fn stage_4_spawn_tile_meshes(
  mut commands: &mut Commands,
  settings: &Res<Settings>,
  resources: &GenerationResourcesCollection,
  mut chunk_entity_pairs: Vec<(Chunk, Entity)>,
  meshes: &mut ResMut<Assets<Mesh>>,
  materials: &mut ResMut<Assets<ColorMaterial>>,
  cg: &Point<ChunkGrid>,
) -> GenerationStage {
  if !chunk_entity_pairs.is_empty() {
    let mut new_chunk_entity_pairs = Vec::new();
    for (chunk, chunk_entity) in chunk_entity_pairs.drain(..) {
      if commands.get_entity(chunk_entity).is_ok() {
        world::spawn_tiles(
          &mut commands,
          chunk_entity,
          chunk.clone(),
          &settings,
          &resources,
          meshes,
          materials,
        );
        new_chunk_entity_pairs.push((chunk, chunk_entity));
      } else {
        trace!(
          "World generation component {cg} - Stage 4 | Chunk entity {:?} at {} no longer exists (it may have been pruned) - skipped scheduling of tile spawning tasks...",
          chunk_entity, chunk.coords.chunk_grid
        );
      }
    }
    return GenerationStage::Stage5(new_chunk_entity_pairs);
  }

  warn!(
    "World generation component {cg} - Stage 4 | No chunk-entity pairs provided - assuming world generation component is redundant..."
  );
  GenerationStage::Stage9
}

fn stage_5_schedule_object_grid_generation(
  commands: &mut Commands,
  settings: &Settings,
  resources: &GenerationResourcesCollection,
  mut chunk_entity_pairs: Vec<(Chunk, Entity)>,
  cg: &Point<ChunkGrid>,
) -> GenerationStage {
  if !chunk_entity_pairs.is_empty() {
    chunk_entity_pairs.retain(|(_, chunk_entity)| commands.get_entity(*chunk_entity).is_ok());
    let settings = settings.clone();
    let resources = resources.clone();
    let task_pool = AsyncComputeTaskPool::get();
    let task = task_pool.spawn(async move {
      let mut triplets = Vec::new();
      for (chunk, chunk_entity) in chunk_entity_pairs.drain(..) {
        let resources = resources.clone();
        let settings = settings.clone();
        if let Some(triplet) = object::generate_object_grid(&resources, &settings, chunk, chunk_entity) {
          triplets.push(triplet);
        }
      }

      triplets
    });
    trace!("World generation component {cg} - Stage 5 | Object grid generation task was scheduled",);
    return GenerationStage::Stage6(task);
  }

  warn!(
    "World generation component {cg} - Stage 5 | No chunk-entity pairs provided - assuming world generation component is redundant..."
  );
  GenerationStage::Stage9
}

fn stage_6_schedule_path_generation(
  commands: &mut Commands,
  metadata: &Metadata,
  object_grid_generation_task: Task<Vec<(Chunk, Entity, ObjectGrid)>>,
  cg: &Point<ChunkGrid>,
) -> GenerationStage {
  if object_grid_generation_task.is_finished() {
    return if let Some(mut triplets) = block_on(poll_once(object_grid_generation_task)) {
      triplets.retain(|(_, chunk_entity, _)| commands.get_entity(*chunk_entity).is_ok());
      let metadata = metadata.clone();
      let task_pool = AsyncComputeTaskPool::get();
      let task = task_pool.spawn(async move {
        let mut new_chunk_entity_grid_triplets = Vec::new();
        for (chunk, chunk_entity, object_grid) in triplets.drain(..) {
          let processed_object_grid = path::calculate_paths(&metadata, object_grid);
          new_chunk_entity_grid_triplets.push((chunk, chunk_entity, processed_object_grid))
        }

        new_chunk_entity_grid_triplets
      });

      trace!("World generation component {cg} - Stage 6 | Path generation task was scheduled");
      return GenerationStage::Stage7(task);
    } else {
      trace!(
        "World generation component {cg} - Stage 6 | Object grid generation task did not return any data - skipping to clean up stage..."
      );

      GenerationStage::Stage9
    };
  }

  GenerationStage::Stage6(object_grid_generation_task)
}

fn stage_7_schedule_generating_object_data(
  commands: &mut Commands,
  settings: &Settings,
  path_generation_task: Task<Vec<(Chunk, Entity, ObjectGrid)>>,
  cg: &Point<ChunkGrid>,
) -> GenerationStage {
  if path_generation_task.is_finished() {
    let mut object_generation_tasks = Vec::new();
    return if let Some(mut triplets) = block_on(poll_once(path_generation_task)) {
      for (chunk, chunk_entity, object_grid) in triplets.drain(..) {
        if commands.get_entity(chunk_entity).is_ok() {
          let settings = settings.clone();
          let task_pool = AsyncComputeTaskPool::get();
          let task =
            task_pool.spawn(async move { object::generate_object_data(&settings, object_grid, chunk, chunk_entity) });
          object_generation_tasks.push(task);
        } else {
          trace!(
            "World generation component {cg} - Stage 7 | Chunk entity {:?} at {} no longer exists (it may have been pruned) - skipped scheduling object data generation...",
            chunk_entity, chunk.coords.chunk_grid
          );
        }
      }

      trace!(
        "World generation component {cg} - Stage 7 | {} object generation tasks were scheduled",
        object_generation_tasks.len()
      );
      return GenerationStage::Stage8(object_generation_tasks);
    } else {
      trace!(
        "World generation component {cg} - Stage 7 | Path generation task did not return any data - skipping to clean up stage..."
      );

      GenerationStage::Stage9
    };
  }

  GenerationStage::Stage7(path_generation_task)
}

fn stage_8_schedule_spawning_objects(
  mut commands: &mut Commands,
  settings: &Settings,
  mut object_generation_task: Vec<Task<Vec<ObjectData>>>,
  cg: &Point<ChunkGrid>,
) -> GenerationStage {
  if !object_generation_task.is_empty() {
    object_generation_task.retain_mut(|task| {
      if task.is_finished() {
        let object_data = block_on(poll_once(task)).expect("Failed to get object data");
        let mut rng = StdRng::seed_from_u64(shared::calculate_seed(*cg, settings.world.noise_seed));
        object::schedule_spawning_objects(&mut commands, &settings, &mut rng, object_data);

        false
      } else {
        true
      }
    });
  }

  if object_generation_task.is_empty() {
    trace!("World generation component {cg} - Stage 8 | No object generation tasks left - marking stage as complete...");
    GenerationStage::Stage9
  } else {
    trace!(
      "World generation component {cg} - Stage 8 | There are still object generation tasks left, so stage is not changing..."
    );
    GenerationStage::Stage8(object_generation_task)
  }
}

fn stage_9_clean_up(
  commands: &mut Commands,
  component: &mut Mut<WorldGenerationComponent>,
  settings: &Res<Settings>,
  entity: Entity,
  existing_chunks: &Res<ChunkComponentIndex>,
  prune_world_event: &mut EventWriter<PruneWorldEvent>,
) -> GenerationStage {
  info!(
    "✅  World generation component {} successfully processed in {} ms",
    component.cg,
    shared::get_time() - component.created_at
  );
  if existing_chunks.size() > MAX_CHUNKS && !component.suppress_pruning_world && settings.general.enable_world_pruning {
    prune_world_event.write(PruneWorldEvent {
      despawn_all_chunks: false,
      update_world_after: false,
    });
  }
  commands.entity(entity).despawn();

  GenerationStage::Done
}

/// Sets the [`GenerationState`] to [`GenerationState::Idling`] when the last [`WorldGenerationComponent`] has just
/// been removed.
fn on_remove_world_generation_component_trigger(
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
    update_world_event.write(event);
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
  identify_chunks_to_despawn(existing_chunks, current_chunk, despawn_all_chunks)
    .iter()
    .for_each(|chunk_entity| {
      if let Ok(mut entity) = commands.get_entity(*chunk_entity) {
        entity.try_despawn();
      }
    });
  info!(
    "World pruning (despawn_all_chunks={}, update_world_after={}) took {} ms on {}",
    despawn_all_chunks,
    update_world_after,
    shared::get_time() - start_time,
    shared::thread_name()
  );
}

fn identify_chunks_to_despawn(
  existing_chunks: &Query<(Entity, &ChunkComponent), With<ChunkComponent>>,
  current_chunk: &Res<CurrentChunk>,
  despawn_all_chunks: bool,
) -> Vec<Entity> {
  let mut chunks_to_despawn = Vec::new();
  for (entity, chunk_component) in existing_chunks.iter() {
    // Case 1: Add chunk if despawn_all_chunks is true
    if despawn_all_chunks {
      trace!(
        "Despawning chunk at {:?} because all chunks have to be despawned",
        chunk_component.coords.chunk_grid
      );
      chunks_to_despawn.push(entity);
      continue;
    }

    // Case 2: Add chunk if it's further away than DESPAWN_DISTANCE
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

    // Case 3: Add chunk if it's a duplicate
    let chunks_with_same_cg: Vec<(Entity, &ChunkComponent)> = existing_chunks
      .iter()
      .filter(|(_, c)| c.coords.chunk_grid == chunk_component.coords.chunk_grid)
      .collect();
    if chunks_with_same_cg.len() > 1 {
      for (duplicate_entity, duplicate_component) in chunks_with_same_cg {
        if chunks_to_despawn.contains(&entity) || duplicate_entity == entity {
          continue;
        }
        chunks_to_despawn.push(duplicate_entity);
        info!(
          "Despawning chunk at {:?} (entity {duplicate_entity}) because it's a duplicate",
          duplicate_component.coords.chunk_grid
        );
      }
    }
  }

  chunks_to_despawn
}
