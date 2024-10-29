use crate::components::{AnimationComponent, AnimationTimer};
use crate::constants::{
  ANIMATION_LENGTH, CHUNK_SIZE, DEFAULT_ANIMATION_FRAME_DURATION, ORIGIN_WORLD_GRID_SPAWN_POINT, TERRAIN_TYPE_ERROR,
};
use crate::coords::point::World;
use crate::coords::Point;
use crate::generation::async_utils::AsyncTask;
use crate::generation::lib::{
  get_direction_points, Chunk, ChunkComponent, DraftChunk, TerrainType, Tile, TileComponent, TileData, WorldComponent,
};
use crate::generation::resources::{AssetPack, GenerationResourcesCollection};
use crate::generation::world::pre_render_processor;
use crate::generation::{async_utils, get_time};
use crate::resources::Settings;
use bevy::app::{App, Plugin, Update};
use bevy::core::Name;
use bevy::ecs::world::CommandQueue;
use bevy::hierarchy::{BuildChildren, BuildWorldChildren, ChildBuilder, WorldChildBuilder};
use bevy::log::*;
use bevy::prelude::{
  Commands, Component, Entity, Query, Res, SpatialBundle, Sprite, SpriteBundle, TextureAtlas, Timer, TimerMode, Transform,
};
use bevy::sprite::Anchor;
use bevy::tasks;
use bevy::tasks::{block_on, AsyncComputeTaskPool, Task};

pub struct WorldGeneratorPlugin;

impl Plugin for WorldGeneratorPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Update, process_async_tasks_system);
  }
}

#[derive(Component)]
struct TileSpawnTask(Task<CommandQueue>);

impl AsyncTask for TileSpawnTask {
  fn poll_once(&mut self) -> Option<CommandQueue> {
    block_on(tasks::poll_once(&mut self.0))
  }
}

pub fn generate_world(mut commands: &mut Commands, settings: &Res<Settings>) -> Vec<(Chunk, Vec<TileData>)> {
  let draft_chunks = generate_draft_chunks(&settings);
  let mut chunks = convert_draft_chunks_to_chunks(&settings, draft_chunks);
  chunks = pre_render_processor::process_all(chunks, &settings);
  let spawn_data = spawn_world_and_chunk_entities(commands, &chunks);
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

fn spawn_world_and_chunk_entities(commands: &mut Commands, chunks: &Vec<Chunk>) -> Vec<(Chunk, Vec<TileData>)> {
  let start_time = get_time();
  let mut spawn_data: Vec<(Chunk, Vec<TileData>)> = Vec::new();
  commands
    .spawn((Name::new("World"), SpatialBundle::default(), WorldComponent))
    .with_children(|parent| {
      for chunk in chunks.iter() {
        let tile_data = spawn_chunk(parent, &chunk);
        spawn_data.push((chunk.clone(), tile_data));
      }
    });
  debug!("Spawned world and chunk entities in {} ms", get_time() - start_time);

  spawn_data
}

pub fn spawn_chunk(world_child_builder: &mut ChildBuilder, chunk: &Chunk) -> Vec<TileData> {
  let mut tile_data = Vec::new();
  let chunk_end_wg = chunk.coords.world_grid + Point::new(CHUNK_SIZE - 1, -CHUNK_SIZE + 1);
  world_child_builder
    .spawn((
      Name::new(format!(
        "Chunk w{} wg{} to wg{}",
        chunk.coords.world, chunk.coords.world_grid, chunk_end_wg
      )),
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
  settings_ref: &Settings,
  spawn_data: &[(Chunk, Vec<TileData>)],
) {
  let start_time = get_time();
  let task_pool = AsyncComputeTaskPool::get();
  for (chunk, tile_data_vec) in spawn_data.iter() {
    for tile_data in tile_data_vec {
      let tile_data = tile_data.clone();
      for layer in 0..TerrainType::length() {
        if layer < settings_ref.general.spawn_from_layer || layer > settings_ref.general.spawn_up_to_layer {
          trace!(
            "Skipped spawning [{:?}] tiles because it's disabled",
            TerrainType::from(layer)
          );
          continue;
        }
        if let Some(plane) = chunk.layered_plane.get(layer) {
          if let Some(tile) = plane.get_tile(tile_data.flat_tile.coords.chunk_grid) {
            if let Some(mut tile_entity) = commands.get_entity(tile_data.entity) {
              tile_entity.with_children(|parent| {
                attach_task_to_tile_entity(task_pool, tile_data, tile.clone(), parent);
              });
            }
          }
        }
      }
    }
  }
  trace!("Scheduled spawning all tiles in {} ms", get_time() - start_time);
}

fn attach_task_to_tile_entity(task_pool: &AsyncComputeTaskPool, tile_data: TileData, tile: Tile, parent: &mut ChildBuilder) {
  let task = task_pool.spawn(async move {
    let mut command_queue = CommandQueue::default();
    command_queue.push(move |world: &mut bevy::prelude::World| {
      let (resources, settings) = async_utils::get_resources_and_settings(world);
      if let Some(mut tile_data_entity) = world.get_entity_mut(tile_data.entity) {
        tile_data_entity.with_children(|parent| {
          spawn_tile(tile_data, &tile, &resources, settings, parent);
        });
      }
    });
    command_queue
  });
  parent.spawn((Name::new("Tile Spawn Task"), TileSpawnTask(task)));
}

fn resolve_asset_pack<'a>(tile: &Tile, resources: &'a GenerationResourcesCollection) -> (bool, &'a AssetPack) {
  let asset_collection = resources.get_terrain_collection(tile.terrain);
  if asset_collection.animated_tile_types.contains(&tile.tile_type) {
    (
      true,
      &asset_collection
        .anim
        .as_ref()
        .expect("Failed to get animated asset pack from resource collection"),
    )
  } else {
    (false, &asset_collection.stat)
  }
}

fn spawn_tile(
  tile_data: TileData,
  tile: &Tile,
  resources: &GenerationResourcesCollection,
  settings: Settings,
  parent: &mut WorldChildBuilder,
) {
  if !settings.general.draw_terrain_sprites {
    parent.spawn(placeholder_sprite(&tile, tile_data.chunk_entity, &resources));
    return;
  }
  if settings.general.animate_terrain_sprites {
    let (is_animated_tile, anim_asset_pack) = resolve_asset_pack(&tile, &resources);
    if is_animated_tile {
      parent.spawn(animated_terrain_sprite(&tile, tile_data.chunk_entity, &anim_asset_pack));
    } else {
      parent.spawn(static_terrain_sprite(&tile, tile_data.chunk_entity, &resources));
    }
  } else {
    parent.spawn(static_terrain_sprite(&tile, tile_data.chunk_entity, &resources));
  }
}

fn placeholder_sprite(
  tile: &Tile,
  chunk: Entity,
  resources: &GenerationResourcesCollection,
) -> (Name, SpriteBundle, TextureAtlas, TileComponent) {
  (
    Name::new(format!("Placeholder {:?} Sprite", tile.terrain)),
    SpriteBundle {
      sprite: Sprite {
        anchor: Anchor::TopLeft,
        ..Default::default()
      },
      texture: resources.placeholder.texture.clone(),
      transform: Transform::from_xyz(0.0, 0.0, tile.layer as f32),
      ..Default::default()
    },
    TextureAtlas {
      layout: resources.placeholder.texture_atlas_layout.clone(),
      index: tile.terrain as usize,
    },
    TileComponent {
      tile: tile.clone(),
      parent_entity: chunk,
    },
  )
}

fn static_terrain_sprite(
  tile: &Tile,
  chunk: Entity,
  resources: &GenerationResourcesCollection,
) -> (Name, SpriteBundle, TextureAtlas, TileComponent) {
  (
    Name::new(format!("{:?} {:?} Sprite", tile.tile_type, tile.terrain)),
    SpriteBundle {
      sprite: Sprite {
        anchor: Anchor::TopLeft,
        ..Default::default()
      },
      texture: match tile.terrain {
        TerrainType::Water => resources.water.stat.texture.clone(),
        TerrainType::Shore => resources.shore.stat.texture.clone(),
        TerrainType::Sand => resources.sand.stat.texture.clone(),
        TerrainType::Grass => resources.grass.stat.texture.clone(),
        TerrainType::Forest => resources.forest.stat.texture.clone(),
        TerrainType::Any => panic!("{}", TERRAIN_TYPE_ERROR),
      },
      transform: Transform::from_xyz(0.0, 0.0, tile.layer as f32),
      ..Default::default()
    },
    TextureAtlas {
      layout: match tile.terrain {
        TerrainType::Water => resources.water.stat.texture_atlas_layout.clone(),
        TerrainType::Shore => resources.shore.stat.texture_atlas_layout.clone(),
        TerrainType::Sand => resources.sand.stat.texture_atlas_layout.clone(),
        TerrainType::Grass => resources.grass.stat.texture_atlas_layout.clone(),
        TerrainType::Forest => resources.forest.stat.texture_atlas_layout.clone(),
        TerrainType::Any => panic!("{}", TERRAIN_TYPE_ERROR),
      },
      index: tile.tile_type.calculate_sprite_index(&tile.terrain, &resources),
    },
    TileComponent {
      tile: tile.clone(),
      parent_entity: chunk,
    },
  )
}

fn animated_terrain_sprite(
  tile: &Tile,
  chunk: Entity,
  asset_pack: &AssetPack,
) -> (Name, SpriteBundle, TextureAtlas, TileComponent, AnimationComponent) {
  let index = tile.tile_type.get_sprite_index(asset_pack.index_offset);
  let frame_duration = match tile.terrain {
    TerrainType::Shore => DEFAULT_ANIMATION_FRAME_DURATION / 2.,
    _ => DEFAULT_ANIMATION_FRAME_DURATION,
  };
  (
    Name::new(format!("{:?} {:?} Sprite (Animated)", tile.tile_type, tile.terrain)),
    SpriteBundle {
      sprite: Sprite {
        anchor: Anchor::TopLeft,
        ..Default::default()
      },
      texture: asset_pack.texture.clone(),
      transform: Transform::from_xyz(0.0, 0.0, tile.layer as f32),
      ..Default::default()
    },
    TextureAtlas {
      layout: asset_pack.texture_atlas_layout.clone(),
      index,
    },
    TileComponent {
      tile: tile.clone(),
      parent_entity: chunk,
    },
    AnimationComponent {
      index_first: index,
      index_last: index + ANIMATION_LENGTH - 1,
      timer: AnimationTimer(Timer::from_seconds(frame_duration, TimerMode::Repeating)),
    },
  )
}

fn process_async_tasks_system(commands: Commands, tile_spawn_tasks: Query<(Entity, &mut TileSpawnTask)>) {
  async_utils::process_tasks(commands, tile_spawn_tasks);
}

pub fn calculate_chunks(chunks_to_spawn: Vec<Point<World>>, settings: &Settings) -> Vec<Chunk> {
  let mut chunks: Vec<Chunk> = Vec::new();
  for chunk_world in chunks_to_spawn {
    let chunk_world_grid = Point::new_world_grid_from_world(chunk_world.clone());
    let draft_chunk = DraftChunk::new(chunk_world_grid, &settings);
    let mut chunk = Chunk::new(draft_chunk, &settings);
    chunk = pre_render_processor::process_single(chunk, &settings);
    chunks.push(chunk);
  }

  chunks
}
