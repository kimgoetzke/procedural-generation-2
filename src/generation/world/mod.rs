use crate::constants::ORIGIN_WORLD_GRID_SPAWN_POINT;
use crate::coords::Point;
use crate::generation::chunk::Chunk;
use crate::generation::components::{ChunkComponent, TileComponent, WorldComponent};
use crate::generation::direction::get_direction_points;
use crate::generation::draft_chunk::DraftChunk;
use crate::generation::get_time;
use crate::generation::resources::AssetPacks;
use crate::generation::terrain_type::TerrainType;
use crate::generation::tile::Tile;
use crate::generation::tile_data::TileData;
use crate::generation::tile_type::get_sprite_index;
use crate::generation::world::pre_render_processor::PreRenderProcessorPlugin;
use crate::resources::Settings;
use bevy::app::{App, Plugin, Update};
use bevy::core::Name;
use bevy::ecs::system::SystemState;
use bevy::ecs::world::CommandQueue;
use bevy::hierarchy::ChildBuilder;
use bevy::log::*;
use bevy::prelude::{
  BuildChildren, BuildWorldChildren, Commands, Component, DespawnRecursiveExt, Entity, Query, Res, SpatialBundle,
  SpriteBundle, TextureAtlas, Transform, World,
};
use bevy::tasks::futures_lite::future;
use bevy::tasks::{block_on, AsyncComputeTaskPool, Task};

pub mod pre_render_processor;

pub struct WorldGenerationPlugin;

impl Plugin for WorldGenerationPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_plugins(PreRenderProcessorPlugin)
      .add_systems(Update, (process_async_tasks_system,));
  }
}

#[derive(Component)]
struct TileSpawnTask(Task<CommandQueue>);

pub fn spawn_world(mut commands: &mut Commands, settings: &Res<Settings>) -> Vec<(Chunk, Vec<TileData>)> {
  let draft_chunks = generate_draft_chunks(&settings);
  let mut final_chunks = convert_draft_chunks_to_chunks(&settings, draft_chunks);
  final_chunks = pre_render_processor::process(final_chunks, &settings);
  let spawn_data = spawn_world_and_chunk_entities(commands, &final_chunks);
  schedule_tile_spawning_tasks(&mut commands, &settings, &spawn_data);

  spawn_data
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

fn spawn_world_and_chunk_entities(commands: &mut Commands, final_chunks: &Vec<Chunk>) -> Vec<(Chunk, Vec<TileData>)> {
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

pub fn schedule_tile_spawning_tasks(
  commands: &mut Commands,
  settings_ref: &Res<Settings>,
  spawn_data: &[(Chunk, Vec<TileData>)],
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

pub fn generate_chunks(
  mut commands: &mut Commands,
  world: Entity,
  chunks_to_spawn: Vec<Point>,
  settings: &Res<Settings>,
) -> Vec<(Chunk, Vec<TileData>)> {
  let mut spawn_data = Vec::new();
  commands.entity(world).with_children(|parent| {
    for chunk_world in chunks_to_spawn.iter() {
      let chunk_world_grid = Point::new_world_grid_from_world(chunk_world.clone());
      let draft_chunk = DraftChunk::new(chunk_world_grid, &settings);
      let chunk = Chunk::new(draft_chunk, settings);
      let tile_data = spawn_chunk(parent, &chunk);
      spawn_data.push((chunk, tile_data));
    }
  });
  schedule_tile_spawning_tasks(&mut commands, &settings, &spawn_data);

  spawn_data
}
