use crate::settings::{
  CHUNK_SIZE, FOREST_TILE, GRASS_TILE, SAND_TILE, TILE_SET_TEST_COLUMNS, TILE_SET_TEST_PATH, TILE_SET_TEST_ROWS,
  TILE_SIZE, WATER_TILE,
};
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

#[derive(Resource)]
struct GeneratedChunks {
  chunks: Vec<Chunk>,
}

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
  let timestamp = SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_millis();
  let texture = asset_server.load(TILE_SET_TEST_PATH);
  let layout = TextureAtlasLayout::from_grid(
    UVec2::splat(TILE_SIZE),
    TILE_SET_TEST_COLUMNS,
    TILE_SET_TEST_ROWS,
    None,
    None,
  );
  let texture_atlas_layout = texture_atlas_layouts.add(layout);

  commands
    .spawn((Name::new("World - Layer 0"), SpatialBundle::default()))
    .with_children(|parent| {
      // Generate data for the initial chunk
      let chunk = generate_chunk_data(seed, Point::new(0, 0));
      let mut chunks: Vec<Chunk> = vec![chunk.clone()];

      // Generate data for the neighbouring chunks
      get_chunk_neighbour_points(chunk).iter().for_each(|point| {
        chunks.push(generate_chunk_data(seed, point.clone()));
      });

      // Spawn all chunks
      for chunk in chunks {
        spawn_chunk(texture.clone(), texture_atlas_layout.clone(), parent, chunk);
      }
    });

  info!(
    "✅  World generation took {}ms",
    SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_millis()
      - timestamp
  );
}

fn spawn_chunk(
  texture: Handle<Image>,
  texture_atlas_layout: Handle<TextureAtlasLayout>,
  world_child_builder: &mut ChildBuilder,
  chunk: Chunk,
) {
  world_child_builder
    .spawn((
      Name::new(format!("Chunk ({},{})", chunk.coords.world.x, chunk.coords.world.y)),
      SpatialBundle::default(),
    ))
    .with_children(|parent| {
      for tile in chunk.tiles.iter() {
        spawn_tile(texture.clone(), texture_atlas_layout.clone(), parent, &tile);
      }
    });
}

fn spawn_tile(
  texture: Handle<Image>,
  texture_atlas_layout: Handle<TextureAtlasLayout>,
  chunk_child_builder: &mut ChildBuilder,
  tile: &Tile,
) {
  chunk_child_builder.spawn((
    Name::new("Tile (".to_string() + &tile.coords.grid.x.to_string() + "," + &tile.coords.grid.y.to_string() + ")"),
    SpriteBundle {
      texture: texture.clone(),
      transform: Transform::from_xyz(
        tile.coords.world.x as f32,
        tile.coords.world.y as f32,
        tile.layer as f32,
      ),
      ..Default::default()
    },
    TextureAtlas {
      layout: texture_atlas_layout.clone(),
      index: tile.sprite_index,
    },
    TileComponent,
  ));
}

fn generate_chunk_data(seed: u32, start: Point) -> Chunk {
  debug!("Generating chunk at {:?}", start);
  let mut tiles = HashSet::new();
  let perlin = Perlin::new(seed);
  let end = Point::new(start.x + CHUNK_SIZE - 1, start.y + CHUNK_SIZE - 1);
  for x in start.x..end.x {
    for y in (start.y..end.y).rev() {
      let noise = perlin.get([x as f64 / CHUNK_SIZE as f64, y as f64 / CHUNK_SIZE as f64]);
      let terrain_type = match noise {
        n if n > 0.7 => TerrainType::Forest,
        n if n > 0.5 => TerrainType::Grass,
        n if n > 0.3 => TerrainType::Sand,
        _ => TerrainType::Water,
      };
      let sprite_index = match terrain_type {
        TerrainType::None | TerrainType::Water => WATER_TILE,
        TerrainType::Sand => SAND_TILE,
        TerrainType::Grass => GRASS_TILE,
        TerrainType::Forest => FOREST_TILE,
      };
      let grid_location = Point::new(x, y);
      let tile = Tile::new(grid_location.clone(), terrain_type, sprite_index, 0);
      trace!("{:?} => Noise: {}", &tile, noise);
      tiles.insert(tile);
    }
  }

  Chunk::new(start, tiles)
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
