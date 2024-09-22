use crate::constants::*;
use crate::coords::Point;
use crate::events::RefreshWorldEvent;
use crate::resources::Settings;
use crate::world::chunk::{get_chunk_spawn_points, Chunk};
use crate::world::components::{ChunkComponent, TileComponent};
use crate::world::draft_chunk::DraftChunk;
use crate::world::resources::WorldResourcesPlugin;
use crate::world::terrain_type::TerrainType;
use crate::world::tile_debugger::TileDebuggerPlugin;
use crate::world::tile_type::*;
use bevy::app::{App, Plugin, Startup};
use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use resources::AssetPacks;
use std::time::SystemTime;
use tile::Tile;

mod chunk;
mod components;
mod draft_chunk;
mod layered_plane;
mod neighbours;
mod plane;
mod resources;
mod terrain_type;
mod tile;
mod tile_debugger;
mod tile_type;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_plugins((TileDebuggerPlugin, WorldResourcesPlugin))
      .add_systems(Startup, generate_world_system)
      .add_systems(Update, refresh_world_event);
  }
}

#[derive(Component)]
struct WorldComponent;

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
  spawn_world(&mut commands, asset_packs, &settings);
}

fn spawn_world(commands: &mut Commands, asset_packs: Res<AssetPacks>, settings: &Res<Settings>) {
  let start_time = get_time();
  let draft_chunks = generate_draft_chunks(settings);
  let final_chunks = convert_draft_chunks_to_chunks(settings, draft_chunks);
  let tile_data = spawn_world_and_base_chunks(commands, &final_chunks);
  spawn_tiles(commands, &asset_packs, &settings, final_chunks, tile_data);
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
    // TODO: Add post-processing step that removes single tiles if layer below is not fill
    final_chunks.push(chunk);
  }
  debug!("Converted draft chunk(s) to chunk(s) in {} ms", get_time() - start_time);

  final_chunks
}

fn spawn_world_and_base_chunks(commands: &mut Commands, final_chunks: &Vec<Chunk>) -> Vec<TileData> {
  let start_time = get_time();
  let mut tile_data = Vec::new();
  commands
    .spawn((Name::new("World"), SpatialBundle::default(), WorldComponent))
    .with_children(|parent| {
      for chunk in final_chunks.iter() {
        let entry = spawn_chunk(parent, &chunk);
        tile_data.extend(entry);
      }
    });
  debug!("Spawned world and chunk entities in {} ms", get_time() - start_time);

  tile_data
}

fn spawn_tiles(
  commands: &mut Commands,
  asset_packs: &Res<AssetPacks>,
  settings: &&Res<Settings>,
  final_chunks: Vec<Chunk>,
  tile_data: Vec<TileData>,
) {
  let t1 = get_time();
  for layer in 0..TerrainType::length() {
    let layer_name = TerrainType::from(layer);
    if layer > settings.general.spawn_up_to_layer {
      debug!("Skipped spawning [{:?}] tiles because it's disabled", layer_name);
      continue;
    }
    let t2 = get_time();
    for chunk in final_chunks.iter() {
      if let Some(plane) = chunk.layered_plane.get(layer) {
        for tile in plane.data.iter().flatten() {
          if let Some(tile) = tile {
            let tile_data = tile_data.iter().find(|x| x.tile.coords == tile.coords).unwrap();
            let tile_commands = commands.entity(tile_data.entity);
            spawn_tile(tile_commands, tile, tile_data.parent_entity, &asset_packs, &settings);
          }
        }
      }
    }
    debug!("Spawned [{:?}] tiles within {} ms", layer_name, get_time() - t2);
  }
  debug!("Spawned all tiles within {} ms", get_time() - t1);
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

fn spawn_tile(mut commands: EntityCommands, tile: &Tile, chunk: Entity, asset_packs: &AssetPacks, settings: &Res<Settings>) {
  commands.with_children(|parent| {
    if settings.general.draw_terrain_sprites {
      parent.spawn(terrain_sprite(tile, chunk, asset_packs));
    } else {
      parent.spawn(default_sprite(tile, chunk, asset_packs));
    }
  });
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
      transform: Transform::from_xyz(0.0, 0.0, tile.layer as f32 + 2.),
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
    spawn_world(&mut commands, asset_packs, &settings);
  }
}
