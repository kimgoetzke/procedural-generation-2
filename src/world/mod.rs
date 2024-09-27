use crate::constants::*;
use crate::coords::Point;
use crate::events::RefreshWorldEvent;
use crate::resources::Settings;
use crate::world::chunk::{get_chunk_spawn_points, Chunk};
use crate::world::components::{ChunkComponent, TileComponent};
use crate::world::draft_chunk::DraftChunk;
use crate::world::object_generation::ObjectGenerationPlugin;
use crate::world::pre_processor::PreProcessorPlugin;
use crate::world::resources::WorldResourcesPlugin;
use crate::world::terrain_type::TerrainType;
use crate::world::tile_debugger::TileDebuggerPlugin;
use crate::world::tile_type::*;
use bevy::app::{App, Plugin, Startup};
use bevy::ecs::system::SystemState;
use bevy::ecs::world::CommandQueue;
use bevy::prelude::*;
use bevy::tasks::futures_lite::future;
use bevy::tasks::{block_on, AsyncComputeTaskPool, Task};
use resources::AssetPacks;
use std::time::SystemTime;
use tile::Tile;

mod chunk;
mod components;
mod draft_chunk;
mod layered_plane;
mod neighbours;
mod object_generation;
mod plane;
mod pre_processor;
mod resources;
mod terrain_type;
mod tile;
mod tile_debugger;
mod tile_type;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_plugins((
        WorldResourcesPlugin,
        PreProcessorPlugin,
        ObjectGenerationPlugin,
        TileDebuggerPlugin,
      ))
      .add_systems(Startup, generate_world_system)
      .add_systems(Update, (refresh_world_event, process_async_tasks_system));
  }
}

#[derive(Component)]
struct TileSpawnTask(Task<CommandQueue>);

#[derive(Component)]
struct WorldComponent;

#[derive(Clone, Copy)]
struct TileData {
  entity: Entity,
  parent_entity: Entity,
  tile: Tile,
}

impl TileData {
  fn new(entity: Entity, parent_entity: Entity, tile: Tile) -> Self {
    Self {
      entity,
      parent_entity,
      tile,
    }
  }
}

fn generate_world_system(mut commands: Commands, asset_packs: Res<AssetPacks>, settings: Res<Settings>) {
  spawn_world(&mut commands, asset_packs, settings);
}

fn spawn_world(commands: &mut Commands, asset_packs: Res<AssetPacks>, settings: Res<Settings>) {
  let start_time = get_time();
  let draft_chunks = generate_draft_chunks(&settings);
  let mut final_chunks = convert_draft_chunks_to_chunks(&settings, draft_chunks);
  final_chunks = pre_processor::process(final_chunks, &settings);
  let mut spawn_data = spawn_world_and_base_chunks(commands, &final_chunks);
  object_generation::process(commands, &mut spawn_data, &asset_packs, &settings);
  schedule_tile_spawning_tasks(commands, &settings, spawn_data);
  info!("✅  World generation took {} ms", get_time() - start_time);
}

fn generate_draft_chunks(settings: &Res<Settings>) -> Vec<DraftChunk> {
  let start_time = get_time();
  let mut draft_chunks: Vec<DraftChunk> = Vec::new();
  let spawn_point = Point::new(-(CHUNK_SIZE / 2), -(CHUNK_SIZE / 2));
  get_chunk_spawn_points(&spawn_point, CHUNK_SIZE).iter().for_each(|point| {
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
        let entry = spawn_chunk(parent, &chunk);
        spawn_data.push((chunk.clone(), entry));
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
  let t1 = get_time();
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
  debug!("Scheduled spawning all tiles within {} ms", get_time() - t1);
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

fn get_time() -> u128 {
  SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()
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
  existing_worlds: Query<Entity, With<WorldComponent>>,
  asset_packs: Res<AssetPacks>,
  settings: Res<Settings>,
) {
  let event_count = events.read().count();
  if event_count > 0 {
    for world in existing_worlds.iter() {
      commands.entity(world).despawn_recursive();
    }
    spawn_world(&mut commands, asset_packs, settings);
  }
}
