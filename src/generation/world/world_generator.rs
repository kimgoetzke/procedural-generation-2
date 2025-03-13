use crate::components::{AnimationComponent, AnimationTimer};
use crate::constants::{ANIMATION_LENGTH, CHUNK_SIZE, DEFAULT_ANIMATION_FRAME_DURATION, TERRAIN_TYPE_ERROR};
use crate::coords::point::World;
use crate::coords::Point;
use crate::generation::lib::shared::CommandQueueTask;
use crate::generation::lib::{shared, Chunk, ChunkComponent, TerrainType, Tile, TileComponent};
use crate::generation::resources::{AssetPack, Climate, GenerationResourcesCollection, Metadata};
use crate::generation::world::post_processor;
use crate::resources::Settings;
use bevy::app::{App, Plugin, Update};
use bevy::core::Name;
use bevy::ecs::world::CommandQueue;
use bevy::hierarchy::{BuildChildren, ChildBuild, ChildBuilder, WorldChildBuilder};
use bevy::log::*;
use bevy::prelude::{Commands, Component, Entity, Query, Sprite, TextureAtlas, Timer, TimerMode, Transform, Visibility};
use bevy::sprite::Anchor;
use bevy::tasks;
use bevy::tasks::{block_on, AsyncComputeTaskPool, Task};

pub struct WorldGeneratorPlugin;

impl Plugin for WorldGeneratorPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Update, process_tile_spawn_tasks_system);
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
    let mut chunk = Chunk::new(chunk_w.clone(), chunk_tg, &metadata, &settings);
    chunk = post_processor::process(chunk, &settings);
    chunks.push(chunk);
  }
  debug!(
    "Generated {} chunks in {} ms on {}",
    chunks.len(),
    shared::get_time() - start_time,
    shared::thread_name()
  );

  chunks
}

pub fn spawn_chunk(world_child_builder: &mut ChildBuilder, chunk: &Chunk) -> Entity {
  let chunk_end_tg = chunk.coords.tile_grid + Point::new(CHUNK_SIZE - 1, -CHUNK_SIZE + 1);
  world_child_builder
    .spawn((
      Name::new(format!(
        "Chunk {} {} {} to {}",
        chunk.coords.chunk_grid, chunk.coords.world, chunk.coords.tile_grid, chunk_end_tg
      )),
      Transform::default(),
      Visibility::default(),
      ChunkComponent {
        layered_plane: chunk.layered_plane.clone(),
        coords: chunk.coords.clone(),
      },
    ))
    .id()
}

pub fn schedule_tile_spawning_tasks(commands: &mut Commands, settings: &Settings, chunk: Chunk, chunk_entity: Entity) {
  let start_time = shared::get_time();
  let task_pool = AsyncComputeTaskPool::get();

  for layer in 0..TerrainType::length() {
    if layer < settings.general.spawn_from_layer || layer > settings.general.spawn_up_to_layer {
      trace!(
        "Skipped spawning [{:?}] tiles because it's disabled",
        TerrainType::from(layer)
      );
      continue;
    }
    if let Some(mut parent_chunk_entity) = commands.get_entity(chunk_entity) {
      if let Some(plane) = chunk.layered_plane.get(layer) {
        plane.data.iter().flatten().for_each(|tile| {
          if let Some(tile) = tile {
            let parent_chunk_entity_id = parent_chunk_entity.id();
            parent_chunk_entity.with_children(|parent| {
              attach_sprite_spawning_task_to_chunk_entity(task_pool, parent, parent_chunk_entity_id, tile.clone());
            });
          }
        });
      }
    }
  }
  debug!(
    "Scheduled spawning tiles for chunk {} in {} ms on {}",
    chunk.coords.chunk_grid,
    shared::get_time() - start_time,
    shared::thread_name()
  );
}

fn attach_sprite_spawning_task_to_chunk_entity(
  task_pool: &AsyncComputeTaskPool,
  parent: &mut ChildBuilder,
  chunk_entity: Entity,
  tile: Tile,
) {
  let task = task_pool.spawn(async move {
    let mut command_queue = CommandQueue::default();
    command_queue.push(move |world: &mut bevy::prelude::World| {
      let resources = shared::get_resources_from_world(world);
      let settings = shared::get_settings_from_world(world);
      if let Ok(mut parent_chunk_entity) = world.get_entity_mut(chunk_entity) {
        parent_chunk_entity.with_children(|parent| {
          spawn_tile(chunk_entity, &tile, &resources, settings, parent);
        });
      }
    });

    command_queue
  });
  parent.spawn((Name::new("Tile Spawn Task"), TileSpawnTask(task)));
}

fn spawn_tile(
  parent_entity: Entity,
  tile: &Tile,
  resources: &GenerationResourcesCollection,
  settings: Settings,
  parent: &mut WorldChildBuilder,
) {
  if !settings.general.draw_terrain_sprites {
    parent.spawn(placeholder_sprite(&tile, parent_entity, &resources));
    return;
  }
  if settings.general.animate_terrain_sprites {
    let (is_animated_tile, anim_asset_pack) = resolve_asset_pack(&tile, &resources);
    if is_animated_tile {
      parent.spawn(animated_terrain_sprite(&tile, parent_entity, &anim_asset_pack));
    } else {
      parent.spawn(static_terrain_sprite(&tile, parent_entity, &resources));
    }
  } else {
    parent.spawn(static_terrain_sprite(&tile, parent_entity, &resources));
  }
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

fn placeholder_sprite(
  tile: &Tile,
  chunk: Entity,
  resources: &GenerationResourcesCollection,
) -> (Name, Sprite, Transform, TileComponent, Visibility) {
  (
    Name::new(format!("{} Placeholder {:?} Sprite", tile.coords.tile_grid, tile.terrain)),
    Sprite {
      anchor: Anchor::TopLeft,
      texture_atlas: Some(TextureAtlas {
        layout: resources.placeholder.texture_atlas_layout.clone(),
        index: tile.terrain as usize,
      }),
      image: resources.placeholder.texture.clone(),
      ..Default::default()
    },
    Transform::from_xyz(tile.coords.world.x as f32, tile.coords.world.y as f32, tile.layer as f32),
    TileComponent {
      tile: tile.clone(),
      parent_entity: chunk,
    },
    Visibility::default(),
  )
}

fn static_terrain_sprite(
  tile: &Tile,
  chunk: Entity,
  resources: &GenerationResourcesCollection,
) -> (Name, Transform, Sprite, TileComponent, Visibility) {
  (
    Name::new(format!(
      "{} {:?} {:?} Sprite",
      tile.coords.tile_grid, tile.tile_type, tile.terrain
    )),
    Transform::from_xyz(tile.coords.world.x as f32, tile.coords.world.y as f32, tile.layer as f32),
    Sprite {
      anchor: Anchor::TopLeft,
      texture_atlas: Some(TextureAtlas {
        layout: match (tile.terrain, tile.climate) {
          (TerrainType::DeepWater, _) => resources.deep_water.stat.texture_atlas_layout.clone(),
          (TerrainType::ShallowWater, _) => resources.shallow_water.stat.texture_atlas_layout.clone(),
          (TerrainType::Land1, Climate::Dry) => resources.land_dry_l1.stat.texture_atlas_layout.clone(),
          (TerrainType::Land1, Climate::Moderate) => resources.land_moderate_l1.stat.texture_atlas_layout.clone(),
          (TerrainType::Land1, Climate::Humid) => resources.land_humid_l1.stat.texture_atlas_layout.clone(),
          (TerrainType::Land2, Climate::Dry) => resources.land_dry_l2.stat.texture_atlas_layout.clone(),
          (TerrainType::Land2, Climate::Moderate) => resources.land_moderate_l2.stat.texture_atlas_layout.clone(),
          (TerrainType::Land2, Climate::Humid) => resources.land_humid_l2.stat.texture_atlas_layout.clone(),
          (TerrainType::Land3, Climate::Dry) => resources.land_dry_l3.stat.texture_atlas_layout.clone(),
          (TerrainType::Land3, Climate::Moderate) => resources.land_moderate_l3.stat.texture_atlas_layout.clone(),
          (TerrainType::Land3, Climate::Humid) => resources.land_humid_l3.stat.texture_atlas_layout.clone(),
          (TerrainType::Any, _) => panic!("{}", TERRAIN_TYPE_ERROR),
        },
        index: tile
          .tile_type
          .calculate_sprite_index(&tile.terrain, &tile.climate, &resources),
      }),
      image: match (tile.terrain, tile.climate) {
        (TerrainType::DeepWater, _) => resources.deep_water.stat.texture.clone(),
        (TerrainType::ShallowWater, _) => resources.shallow_water.stat.texture.clone(),
        (TerrainType::Land1, Climate::Dry) => resources.land_dry_l1.stat.texture.clone(),
        (TerrainType::Land1, Climate::Moderate) => resources.land_moderate_l1.stat.texture.clone(),
        (TerrainType::Land1, Climate::Humid) => resources.land_humid_l1.stat.texture.clone(),
        (TerrainType::Land2, Climate::Dry) => resources.land_dry_l2.stat.texture.clone(),
        (TerrainType::Land2, Climate::Moderate) => resources.land_moderate_l2.stat.texture.clone(),
        (TerrainType::Land2, Climate::Humid) => resources.land_humid_l2.stat.texture.clone(),
        (TerrainType::Land3, Climate::Dry) => resources.land_dry_l3.stat.texture.clone(),
        (TerrainType::Land3, Climate::Moderate) => resources.land_moderate_l3.stat.texture.clone(),
        (TerrainType::Land3, Climate::Humid) => resources.land_humid_l3.stat.texture.clone(),
        (TerrainType::Any, _) => panic!("{}", TERRAIN_TYPE_ERROR),
      },
      ..Default::default()
    },
    TileComponent {
      tile: tile.clone(),
      parent_entity: chunk,
    },
    Visibility::default(),
  )
}

fn animated_terrain_sprite(
  tile: &Tile,
  chunk: Entity,
  asset_pack: &AssetPack,
) -> (Name, Transform, Sprite, TileComponent, AnimationComponent, Visibility) {
  let index = tile.tile_type.get_sprite_index(asset_pack.index_offset);
  let frame_duration = match tile.terrain {
    TerrainType::ShallowWater => DEFAULT_ANIMATION_FRAME_DURATION / 2.,
    _ => DEFAULT_ANIMATION_FRAME_DURATION,
  };
  (
    Name::new(format!(
      "{} {:?} {:?} Sprite (Animated)",
      tile.coords.tile_grid, tile.tile_type, tile.terrain
    )),
    Transform::from_xyz(tile.coords.world.x as f32, tile.coords.world.y as f32, tile.layer as f32),
    Sprite {
      anchor: Anchor::TopLeft,
      texture_atlas: Some(TextureAtlas {
        layout: asset_pack.texture_atlas_layout.clone(),
        index,
      }),
      image: asset_pack.texture.clone(),
      ..Default::default()
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
    Visibility::default(),
  )
}

fn process_tile_spawn_tasks_system(commands: Commands, tile_spawn_tasks: Query<(Entity, &mut TileSpawnTask)>) {
  shared::process_tasks(commands, tile_spawn_tasks);
}
