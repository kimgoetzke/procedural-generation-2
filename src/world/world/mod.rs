use crate::constants::{CHUNK_SIZE, ORIGIN_WORLD_GRID_SPAWN_POINT, TILE_SIZE};
use crate::coords::Point;
use crate::events::{ChunkGenerationEvent, DespawnDistantChunkEvent, RefreshWorldEvent};
use crate::resources::{CurrentChunk, Settings};
use crate::world::chunk::Chunk;
use crate::world::components::{ChunkComponent, TileComponent};
use crate::world::direction::get_direction_points;
use crate::world::draft_chunk::DraftChunk;
use crate::world::resources::{AssetPacks, ChunkComponentIndex};
use crate::world::terrain_type::TerrainType;
use crate::world::tile::Tile;
use crate::world::tile_type::get_sprite_index;
use crate::world::{direction, get_time, object, pre_processor, TileData};
use bevy::app::{App, Plugin, Startup, Update};
use bevy::core::Name;
use bevy::ecs::system::SystemState;
use bevy::ecs::world::CommandQueue;
use bevy::hierarchy::ChildBuilder;
use bevy::log::{debug, info};
use bevy::prelude::{
  BuildChildren, BuildWorldChildren, Commands, Component, DespawnRecursiveExt, Entity, EventReader, EventWriter, Query, Res,
  ResMut, SpatialBundle, SpriteBundle, TextureAtlas, Transform, With, World,
};
use bevy::tasks::futures_lite::future;
use bevy::tasks::{block_on, AsyncComputeTaskPool, Task};
use direction::Direction;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Startup, generate_world_system).add_systems(
      Update,
      (
        process_async_tasks_system,
        refresh_world_event,
        chunk_generation_event,
        despawn_distant_chunks_event,
      ),
    );
  }
}

#[derive(Component)]
struct TileSpawnTask(Task<CommandQueue>);

#[derive(Component)]
struct WorldComponent;

fn generate_world_system(mut commands: Commands, asset_packs: Res<AssetPacks>, settings: Res<Settings>) {
  spawn_world(&mut commands, asset_packs, settings);
}

fn spawn_world(commands: &mut Commands, asset_packs: Res<AssetPacks>, settings: Res<Settings>) {
  let start_time = get_time();
  let draft_chunks = generate_draft_chunks(&settings);
  let mut final_chunks = convert_draft_chunks_to_chunks(&settings, draft_chunks);
  final_chunks = pre_processor::process(final_chunks, &settings);
  let mut spawn_data = spawn_world_and_base_chunks(commands, &final_chunks);
  object::generate_objects(commands, &mut spawn_data, &asset_packs, &settings);
  schedule_tile_spawning_tasks(commands, &settings, spawn_data);
  info!("✅  World generation took {} ms", get_time() - start_time);
}

fn generate_draft_chunks(settings: &Res<Settings>) -> Vec<DraftChunk> {
  let start_time = get_time();
  let mut draft_chunks: Vec<DraftChunk> = Vec::new();
  let spawn_point = ORIGIN_WORLD_GRID_SPAWN_POINT;
  get_direction_points(&spawn_point).iter().for_each(|(_, point)| {
    if settings.general.generate_neighbour_chunks {
      let draft_chunk = DraftChunk::new(point.clone(), settings);
      draft_chunks.push(draft_chunk);
    } else {
      if point.x == spawn_point.x && point.y == spawn_point.y {
        debug!("Skipped generating neighbour chunks because it's disabled");
        let draft_chunk = DraftChunk::new(point.clone(), settings);
        draft_chunks.push(draft_chunk);
      }
    }
  });
  debug!("Generated draft chunk(s) in {} ms", get_time() - start_time);

  draft_chunks
}

fn convert_draft_chunks_to_chunks(settings: &Res<Settings>, draft_chunks: Vec<DraftChunk>) -> Vec<Chunk> {
  let start_time = get_time();
  let mut final_chunks: Vec<Chunk> = Vec::new();
  for draft_chunk in draft_chunks {
    let chunk = Chunk::new(draft_chunk, settings);
    final_chunks.push(chunk);
  }
  debug!("Converted draft chunk(s) to chunk(s) in {} ms", get_time() - start_time);

  final_chunks
}

fn spawn_world_and_base_chunks(commands: &mut Commands, final_chunks: &Vec<Chunk>) -> Vec<(Chunk, Vec<TileData>)> {
  let start_time = get_time();
  let mut spawn_data: Vec<(Chunk, Vec<TileData>)> = Vec::new();
  commands
    .spawn((Name::new("World"), SpatialBundle::default(), WorldComponent))
    .with_children(|parent| {
      for chunk in final_chunks.iter() {
        let tile_data = spawn_chunk(parent, &chunk);
        spawn_data.push((chunk.clone(), tile_data));
      }
    });
  debug!("Spawned world and chunk entities in {} ms", get_time() - start_time);

  spawn_data
}

fn spawn_chunk(world_child_builder: &mut ChildBuilder, chunk: &Chunk) -> Vec<TileData> {
  let mut tile_data = Vec::new();
  debug!("Spawning chunk at {:?}", chunk.coords);
  world_child_builder
    .spawn((
      Name::new(format!("Chunk w{}", chunk.coords.world)),
      SpatialBundle::default(),
      ChunkComponent {
        layered_plane: chunk.layered_plane.clone(),
        coords: chunk.coords.clone(),
      },
    ))
    .with_children(|parent| {
      for cell in chunk.layered_plane.flat.data.iter().flatten() {
        if let Some(tile) = cell {
          let tile_entity = parent
            .spawn((
              Name::new("Tile wg".to_string() + &tile.coords.world_grid.to_string()),
              SpatialBundle {
                transform: Transform::from_xyz(tile.coords.world.x as f32, tile.coords.world.y as f32, 0.),
                ..Default::default()
              },
            ))
            .id();
          tile_data.push(TileData::new(tile_entity, parent.parent_entity(), tile.clone()));
        }
      }
    });

  tile_data
}

fn schedule_tile_spawning_tasks(
  commands: &mut Commands,
  settings_ref: &Res<Settings>,
  spawn_data: Vec<(Chunk, Vec<TileData>)>,
) {
  let start_time = get_time();
  let thread_pool = AsyncComputeTaskPool::get();
  for (chunk, tile_data_vec) in spawn_data.iter() {
    for tile_data in tile_data_vec {
      let tile_data = tile_data.clone();
      for layer in 0..TerrainType::length() {
        if layer >= settings_ref.general.spawn_up_to_layer {
          debug!(
            "Skipped spawning [{:?}] tiles because it's disabled",
            TerrainType::from(layer)
          );
          continue;
        }
        if let Some(plane) = chunk.layered_plane.get(layer) {
          if let Some(tile) = plane.get_tile(tile_data.tile.coords.chunk_grid) {
            commands.entity(tile_data.entity).with_children(|parent| {
              attach_task_to_tile_entity(thread_pool, tile_data, tile.clone(), parent);
            });
          }
        }
      }
    }
  }
  debug!("Scheduled spawning all tiles within {} ms", get_time() - start_time);
}

fn attach_task_to_tile_entity(
  thread_pool: &AsyncComputeTaskPool,
  tile_data: TileData,
  tile: Tile,
  parent: &mut ChildBuilder,
) {
  let mut entity = parent.spawn(Name::new("Tile Spawn Task"));
  let task = thread_pool.spawn(async move {
    let mut command_queue = CommandQueue::default();
    command_queue.push(move |world: &mut World| {
      let (asset_packs, settings) = {
        let mut system_state = SystemState::<(Res<AssetPacks>, Res<Settings>)>::new(world);
        let (asset_packs, settings) = system_state.get_mut(world);
        (asset_packs.clone(), settings.clone())
      };
      world.entity_mut(tile_data.entity).with_children(|parent| {
        if settings.general.draw_terrain_sprites {
          parent.spawn(terrain_sprite(&tile, tile_data.parent_entity, &asset_packs));
        } else {
          parent.spawn(default_sprite(&tile, tile_data.parent_entity, &asset_packs));
        }
      });
    });
    command_queue
  });
  entity.insert(TileSpawnTask(task));
}

fn default_sprite(
  tile: &Tile,
  chunk: Entity,
  asset_packs: &AssetPacks,
) -> (Name, SpriteBundle, TextureAtlas, TileComponent) {
  (
    Name::new(format!("Default {:?} Sprite", tile.terrain)),
    SpriteBundle {
      texture: asset_packs.default.texture.clone(),
      transform: Transform::from_xyz(0.0, 0.0, tile.layer as f32),
      ..Default::default()
    },
    TextureAtlas {
      layout: asset_packs.default.texture_atlas_layout.clone(),
      index: tile.terrain as usize,
    },
    TileComponent {
      tile: tile.clone(),
      parent_entity: chunk,
    },
  )
}

// TODO: Add support for animated tile sprites
fn terrain_sprite(
  tile: &Tile,
  chunk: Entity,
  asset_packs: &AssetPacks,
) -> (Name, SpriteBundle, TextureAtlas, TileComponent) {
  (
    Name::new(format!("{:?} {:?} Sprite", tile.tile_type, tile.terrain)),
    SpriteBundle {
      texture: match tile.terrain {
        TerrainType::Water => asset_packs.water.texture.clone(),
        TerrainType::Shore => asset_packs.shore.texture.clone(),
        TerrainType::Sand => asset_packs.sand.texture.clone(),
        TerrainType::Grass => asset_packs.grass.texture.clone(),
        TerrainType::Forest => asset_packs.forest.texture.clone(),
        _ => panic!("Invalid terrain type for drawing a terrain sprite"),
      },
      transform: Transform::from_xyz(0.0, 0.0, tile.layer as f32),
      ..Default::default()
    },
    TextureAtlas {
      layout: match tile.terrain {
        TerrainType::Water => asset_packs.water.texture_atlas_layout.clone(),
        TerrainType::Shore => asset_packs.shore.texture_atlas_layout.clone(),
        TerrainType::Sand => asset_packs.sand.texture_atlas_layout.clone(),
        TerrainType::Grass => asset_packs.grass.texture_atlas_layout.clone(),
        TerrainType::Forest => asset_packs.forest.texture_atlas_layout.clone(),
        _ => panic!("Invalid terrain type for drawing a terrain sprite"),
      },
      index: get_sprite_index(&tile),
    },
    TileComponent {
      tile: tile.clone(),
      parent_entity: chunk,
    },
  )
}

fn process_async_tasks_system(mut commands: Commands, mut transform_tasks: Query<(Entity, &mut TileSpawnTask)>) {
  for (entity, mut task) in &mut transform_tasks {
    if let Some(mut commands_queue) = block_on(future::poll_once(&mut task.0)) {
      commands.append(&mut commands_queue);
      commands.entity(entity).despawn_recursive();
    }
  }
}

fn refresh_world_event(
  mut commands: Commands,
  mut events: EventReader<RefreshWorldEvent>,
  existing_world: Query<Entity, With<WorldComponent>>,
  asset_packs: Res<AssetPacks>,
  settings: Res<Settings>,
) {
  let event_count = events.read().count();
  if event_count > 0 {
    let world = existing_world.get_single().unwrap();
    commands.entity(world).despawn_recursive();
    spawn_world(&mut commands, asset_packs, settings);
  }
}

fn chunk_generation_event(
  mut commands: Commands,
  mut events: EventReader<ChunkGenerationEvent>,
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
      return;
    }

    let current_chunk_world = current_chunk.get_world();
    let direction = Direction::from_chunk(&current_chunk_world, &event_world);
    let new_parent_chunk_world = calculate_new_current_chunk_world(&current_chunk_world, &direction);
    debug!(
      "Received chunk generation event at w{} wg{}; current_parent={}, direction={:?}, new_parent={}",
      event_world, event_world_grid, current_chunk_world, direction, new_parent_chunk_world
    );
    let mut chunks_to_spawn = Vec::new();
    get_direction_points(&new_parent_chunk_world)
      .iter()
      .for_each(|(direction, chunk_world)| {
        if let Some(_) = existing_chunks.get(*chunk_world) {
          debug!("✅  {:?} chunk at w{:?} already exists", direction, chunk_world);
        } else {
          if !settings.general.generate_neighbour_chunks && chunk_world != &new_parent_chunk_world {
            debug!(
              "🚫 {:?} chunk at w{:?} does not exist but skipping because generating neighbours is disabled",
              direction, chunk_world
            );
            return;
          }
          debug!("🚫 {:?} chunk at w{:?} does not exist", direction, chunk_world);
          chunks_to_spawn.push(chunk_world.clone());
        }
      });

    // TODO: Detect current chunk changes when moving to the top or right earlier - its triggered after having left the current chunk
    // TODO: Calculate which chunks need to be despawned and despawn them
    // TODO: Clean up and refactor

    let world = existing_world.get_single().unwrap();
    let mut spawn_data = Vec::new();
    commands.entity(world).with_children(|parent| {
      for chunk_world in chunks_to_spawn.iter() {
        let chunk_world_grid = Point::new_world_grid_from_world(chunk_world.clone());
        let draft_chunk = DraftChunk::new(chunk_world_grid, &settings);
        let final_chunk = convert_draft_chunks_to_chunks(&settings, vec![draft_chunk]);
        let chunk = final_chunk.first().unwrap();
        let tile_data = spawn_chunk(parent, chunk);
        spawn_data.push((chunk.clone(), tile_data));
      }
    });
    object::generate_objects(&mut commands, &mut spawn_data, &asset_packs, &settings);
    schedule_tile_spawning_tasks(&mut commands, &settings, spawn_data);

    current_chunk.update(new_parent_chunk_world);
    despawn_distant_chunk_event.send(DespawnDistantChunkEvent {});
    info!("Chunk generation event took {} ms", get_time() - start_time);
  }
}

pub fn calculate_new_current_chunk_world(current_chunk_world: &Point, direction: &Direction) -> Point {
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
        debug!(
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
    info!("Despawning distant chunks took {} ms", get_time() - start_time);
  }
}
