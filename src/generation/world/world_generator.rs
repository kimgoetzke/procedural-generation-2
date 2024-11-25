use crate::components::{AnimationComponent, AnimationTimer};
use crate::constants::{ANIMATION_LENGTH, CHUNK_SIZE, DEFAULT_ANIMATION_FRAME_DURATION, TERRAIN_TYPE_ERROR};
use crate::coords::point::World;
use crate::coords::Point;
use crate::generation::lib::shared::CommandQueueTask;
use crate::generation::lib::{shared, Chunk, ChunkComponent, DraftChunk, TerrainType, Tile, TileComponent, TileData};
use crate::generation::resources::{AssetPack, Climate, GenerationResourcesCollection, Metadata};
use crate::generation::world::post_processor;
use crate::resources::Settings;
use bevy::app::{App, Plugin, Update};
use bevy::core::Name;
use bevy::ecs::world::CommandQueue;
use bevy::hierarchy::{BuildChildren, BuildWorldChildren, ChildBuilder, WorldChildBuilder};
use bevy::log::*;
use bevy::prelude::{
  Commands, Component, Entity, Query, SpatialBundle, Sprite, SpriteBundle, TextureAtlas, Timer, TimerMode, Transform,
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

impl CommandQueueTask for TileSpawnTask {
  fn poll_once(&mut self) -> Option<CommandQueue> {
    block_on(tasks::poll_once(&mut self.0))
  }
}

pub fn generate_chunks(spawn_points: Vec<Point<World>>, metadata: Metadata, settings: &Settings) -> Vec<Chunk> {
  let start_time = shared::get_time();
  let mut chunks: Vec<Chunk> = Vec::new();
  for chunk_w in spawn_points {
    let chunk_tg = Point::new_tile_grid_from_world(chunk_w.clone());
    let draft_chunk = DraftChunk::new(chunk_w.clone(), chunk_tg, &metadata, &settings);
    let mut chunk = Chunk::new(draft_chunk, &settings);
    chunk = post_processor::process(chunk, &settings);
    chunks.push(chunk);
  }
  debug!(
    "Generated {} chunks in {} ms on [{}]",
    chunks.len(),
    shared::get_time() - start_time,
    shared::thread_name()
  );

  chunks
}

pub fn spawn_chunk(world_child_builder: &mut ChildBuilder, chunk: &Chunk) -> Vec<TileData> {
  let mut tile_data = Vec::new();
  let chunk_end_tg = chunk.coords.tile_grid + Point::new(CHUNK_SIZE - 1, -CHUNK_SIZE + 1);
  world_child_builder
    .spawn((
      Name::new(format!(
        "Chunk {} {} to {}",
        chunk.coords.world, chunk.coords.tile_grid, chunk_end_tg
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
              Name::new("Tile ".to_string() + &tile.coords.tile_grid.to_string()),
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

pub fn schedule_tile_spawning_tasks(commands: &mut Commands, settings: &Settings, spawn_data: (Chunk, Vec<TileData>)) {
  let start_time = shared::get_time();
  let task_pool = AsyncComputeTaskPool::get();

  for tile_data in spawn_data.1 {
    let tile_data = tile_data.clone();
    for layer in 0..TerrainType::length() {
      if layer < settings.general.spawn_from_layer || layer > settings.general.spawn_up_to_layer {
        trace!(
          "Skipped spawning [{:?}] tiles because it's disabled",
          TerrainType::from(layer)
        );
        continue;
      }
      if let Some(plane) = spawn_data.0.layered_plane.get(layer) {
        if let Some(tile) = plane.get_tile(tile_data.flat_tile.coords.internal_grid) {
          if let Some(mut tile_entity) = commands.get_entity(tile_data.entity) {
            tile_entity.with_children(|parent| {
              attach_task_to_tile_entity(task_pool, parent, tile_data, tile.clone());
            });
          }
        }
      }
    }
  }
  debug!(
    "Scheduled spawning tiles for chunk {} in {} ms on [{}]",
    spawn_data.0.coords.chunk_grid,
    shared::get_time() - start_time,
    shared::thread_name()
  );
}

fn attach_task_to_tile_entity(task_pool: &AsyncComputeTaskPool, parent: &mut ChildBuilder, tile_data: TileData, tile: Tile) {
  let task = task_pool.spawn(async move {
    let mut command_queue = CommandQueue::default();
    command_queue.push(move |world: &mut bevy::prelude::World| {
      let (resources, settings) = shared::get_resources_and_settings(world);
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
  let asset_collection = resources.get_terrain_collection(tile.terrain, tile.climate);
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
      texture: match (tile.terrain, tile.climate) {
        (TerrainType::DeepWater, _) => resources.deep_water.stat.texture.clone(),
        (TerrainType::ShallowWater, _) => resources.shallow_water.stat.texture.clone(),
        (TerrainType::Land1, Climate::Dry) => resources.land_dry_l1.stat.texture.clone(),
        (TerrainType::Land1, Climate::Moderate) => resources.land_moderate_l1.stat.texture.clone(),
        (TerrainType::Land2, Climate::Dry) => resources.land_dry_l2.stat.texture.clone(),
        (TerrainType::Land2, Climate::Moderate) => resources.land_moderate_l2.stat.texture.clone(),
        (TerrainType::Land3, Climate::Dry) => resources.land_dry_l3.stat.texture.clone(),
        (TerrainType::Land3, Climate::Moderate) => resources.land_moderate_l3.stat.texture.clone(),
        (TerrainType::Any, _) => panic!("{}", TERRAIN_TYPE_ERROR),
      },
      transform: Transform::from_xyz(0.0, 0.0, tile.layer as f32),
      ..Default::default()
    },
    TextureAtlas {
      layout: match (tile.terrain, tile.climate) {
        (TerrainType::DeepWater, _) => resources.deep_water.stat.texture_atlas_layout.clone(),
        (TerrainType::ShallowWater, _) => resources.shallow_water.stat.texture_atlas_layout.clone(),
        (TerrainType::Land1, Climate::Dry) => resources.land_dry_l1.stat.texture_atlas_layout.clone(),
        (TerrainType::Land1, Climate::Moderate) => resources.land_moderate_l1.stat.texture_atlas_layout.clone(),
        (TerrainType::Land2, Climate::Dry) => resources.land_dry_l2.stat.texture_atlas_layout.clone(),
        (TerrainType::Land2, Climate::Moderate) => resources.land_moderate_l2.stat.texture_atlas_layout.clone(),
        (TerrainType::Land3, Climate::Dry) => resources.land_dry_l3.stat.texture_atlas_layout.clone(),
        (TerrainType::Land3, Climate::Moderate) => resources.land_moderate_l3.stat.texture_atlas_layout.clone(),
        (TerrainType::Any, _) => panic!("{}", TERRAIN_TYPE_ERROR),
      },
      index: tile
        .tile_type
        .calculate_sprite_index(&tile.terrain, &tile.climate, &resources),
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
    TerrainType::ShallowWater => DEFAULT_ANIMATION_FRAME_DURATION / 2.,
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
  shared::process_tasks(commands, tile_spawn_tasks);
}
