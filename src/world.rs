use crate::settings::*;
use crate::shared::*;
use crate::shared_events::RefreshWorldEvent;
use bevy::app::{App, Plugin, Startup};
use bevy::prelude::*;
use bevy::utils::HashSet;
use noise::{NoiseFn, Perlin};
use std::time::SystemTime;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(Startup, generate_world_system)
      .add_systems(Update, refresh_world_event)
      .insert_resource(Seed(1));
  }
}

#[derive(Component)]
struct TileComponent;

#[derive(Resource)]
struct Seed(u32);

fn generate_world_system(
  mut commands: Commands,
  seed: Res<Seed>,
  asset_server: Res<AssetServer>,
  mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
  spawn_world(&mut commands, seed.0, asset_server, &mut texture_atlas_layouts);
}

fn spawn_world(
  commands: &mut Commands,
  seed: u32,
  asset_server: Res<AssetServer>,
  texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
) {
  let timestamp = get_timestamp();
  let asset_packs = get_asset_packs(&asset_server, texture_atlas_layouts);

  commands
    .spawn((Name::new("World - Layer 0"), SpatialBundle::default()))
    .with_children(|parent| {
      // Generate data for the initial chunk
      let chunk = generate_chunk_layer_data(seed, Point::new(0, 0));
      let mut chunks: Vec<Chunk> = vec![chunk.clone()];

      // Generate data for all neighbouring chunks
      get_chunk_neighbour_points(chunk).iter().for_each(|point| {
        chunks.push(generate_chunk_layer_data(seed, point.clone()));
      });

      // Spawn all chunks
      for mut chunk in chunks {
        chunk.determine_tile_types();
        spawn_chunk(&asset_packs, parent, chunk);
      }
    });

  info!("✅  World generation took {} ms", get_timestamp() - timestamp);
}

fn spawn_chunk(asset_packs: &AssetPacks, world_child_builder: &mut ChildBuilder, chunk: Chunk) {
  world_child_builder
    .spawn((
      Name::new(format!("Chunk ({},{})", chunk.coords.world.x, chunk.coords.world.y)),
      SpatialBundle::default(),
    ))
    .with_children(|parent| {
      for (_, tile) in chunk.tiles.iter() {
        spawn_tile(asset_packs, parent, &tile);
      }
    });
}

fn spawn_tile(asset_packs: &AssetPacks, chunk_child_builder: &mut ChildBuilder, tile: &Tile) {
  chunk_child_builder.spawn((
    Name::new("Tile (".to_string() + &tile.coords.grid.x.to_string() + "," + &tile.coords.grid.y.to_string() + ")"),
    SpriteBundle {
      texture: match tile.terrain {
        TerrainType::Sand => asset_packs.sand.texture.clone(),
        _ => asset_packs.default.texture.clone(),
      },
      transform: Transform::from_xyz(
        tile.coords.world.x as f32,
        tile.coords.world.y as f32,
        tile.layer as f32,
      ),
      ..Default::default()
    },
    TextureAtlas {
      layout: match tile.terrain {
        TerrainType::Sand => asset_packs.sand.texture_atlas_layout.clone(),
        _ => asset_packs.default.texture_atlas_layout.clone(),
      },
      index: match tile.terrain {
        TerrainType::Sand => get_sprite_index(&tile),
        _ => tile.default_sprite_index,
      },
    },
    TileComponent,
  ));
}

fn generate_chunk_layer_data(seed: u32, start: Point) -> Chunk {
  let timestamp = get_timestamp();
  let mut tiles = HashSet::new();
  let perlin = Perlin::new(seed);
  let end = Point::new(start.x + CHUNK_SIZE - 1, start.y + CHUNK_SIZE - 1);
  for x in start.x..end.x {
    for y in (start.y..end.y).rev() {
      let noise = perlin.get([x as f64 / CHUNK_SIZE as f64, y as f64 / CHUNK_SIZE as f64]);
      let terrain_type = match noise {
        n if n > 0.9 => TerrainType::Forest,
        n if n > 0.6 => TerrainType::Grass,
        n if n > 0.4 => TerrainType::Sand,
        n if n > 0.1 => TerrainType::Shore,
        _ => TerrainType::Water,
      };
      let tile = match terrain_type {
        TerrainType::Water => Tile::new(Point::new(x, y), terrain_type, WATER_TILE, WATER_TILE as i32),
        TerrainType::Shore => Tile::new(Point::new(x, y), terrain_type, SHORE_TILE, SHORE_TILE as i32),
        TerrainType::Sand => Tile::new(Point::new(x, y), terrain_type, SAND_TILE, SAND_TILE as i32),
        TerrainType::Grass => Tile::new(Point::new(x, y), terrain_type, GRASS_TILE, GRASS_TILE as i32),
        TerrainType::Forest => Tile::new(Point::new(x, y), terrain_type, FOREST_TILE, FOREST_TILE as i32),
      };
      trace!("{:?} => Noise: {}", &tile, noise);
      tiles.insert(tile);
    }
  }
  debug!(
    "✅  Generated chunk layer data for chunk at {:?} within {} ms",
    start,
    get_timestamp() - timestamp
  );

  Chunk::new(start, tiles)
}

fn get_timestamp() -> u128 {
  SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_millis()
}

fn refresh_world_event(
  mut commands: Commands,
  mut events: EventReader<RefreshWorldEvent>,
  existing_tiles: Query<Entity, With<TileComponent>>,
  mut seed: ResMut<Seed>,
  asset_server: Res<AssetServer>,
  mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
  let event_count = events.read().count();
  if event_count > 0 {
    for entity in existing_tiles.iter() {
      commands.entity(entity).despawn();
    }
    seed.0 += 1;
    spawn_world(&mut commands, seed.0, asset_server, &mut texture_atlas_layouts);
  }
}
