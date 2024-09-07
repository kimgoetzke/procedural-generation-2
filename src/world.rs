use crate::settings::{
  CHUNK_SIZE, FOREST_TILE, GRASS_TILE, SAND_TILE, TILE_SET_DEFAULT_COLUMNS, TILE_SET_DEFAULT_PATH,
  TILE_SET_DEFAULT_ROWS, TILE_SET_TEST_COLUMNS, TILE_SET_TEST_PATH, TILE_SET_TEST_ROWS, TILE_SIZE, WATER_TILE,
};
use crate::shared_events::RefreshWorldEvent;
use bevy::app::{App, Plugin, Startup};
use bevy::prelude::*;
use bevy::scene::ron::de::Position;
use bevy::utils::{HashMap, HashSet};
use noise::{NoiseFn, Perlin};
use rand::{random, Rng};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(Startup, generate_world_system)
      .add_systems(Update, refresh_world_event)
      .insert_resource(GroundTiles(HashSet::new()))
      .insert_resource(Seed(1));
  }
}

#[derive(Debug, Eq, PartialEq, Hash)]
struct Tile {
  position_grid: Point,
  position_world: Point,
  sprite_index: usize,
  z_index: i32,
}

impl Tile {
  fn new(grid_location: Point, sprite_index: usize, z_index: i32) -> Self {
    Self {
      position_grid: grid_location.clone(),
      position_world: Point::new(grid_location.x * TILE_SIZE as i32, grid_location.y * TILE_SIZE as i32),
      sprite_index,
      z_index,
    }
  }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct Point {
  x: i32,
  y: i32,
}

impl Point {
  fn new(x: i32, y: i32) -> Self {
    Self { x, y }
  }
}

#[derive(Component)]
struct TileComponent;

#[derive(Resource)]
pub struct GroundTiles(pub HashSet<(i32, i32)>);

#[derive(Resource)]
struct Seed(u32);

fn generate_world_system(
  mut commands: Commands,
  seed: Res<Seed>,
  asset_server: Res<AssetServer>,
  mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
  generate_world(&mut commands, seed.0, asset_server, &mut texture_atlas_layouts);
}

fn generate_world(
  commands: &mut Commands,
  seed: u32,
  asset_server: Res<AssetServer>,
  texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
) {
  let texture = asset_server.load(TILE_SET_TEST_PATH);
  let layout = TextureAtlasLayout::from_grid(
    UVec2::splat(TILE_SIZE),
    TILE_SET_TEST_COLUMNS,
    TILE_SET_TEST_ROWS,
    None,
    None,
  );
  let texture_atlas_layout = texture_atlas_layouts.add(layout);
  let world = commands.spawn((Name::new("World - Layer 0"), SpatialBundle::default())).id();
  let tiles = generate_chunk(seed, Point::new(0, 0));

  commands.entity(world).with_children(|parent| {
    for tile in tiles.iter() {
      parent.spawn((
        Name::new(
          "Tile (".to_string() + &tile.position_grid.x.to_string() + "," + &tile.position_grid.y.to_string() + ")",
        ),
        SpriteBundle {
          texture: texture.clone(),
          transform: Transform::from_xyz(
            tile.position_world.x as f32,
            tile.position_world.y as f32,
            tile.z_index as f32,
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
  });
}

fn generate_chunk(seed: u32, start: Point) -> HashSet<Tile> {
  let mut tiles = HashSet::new();
  let perlin = Perlin::new(seed);
  let end = Point::new(start.x + CHUNK_SIZE - 1, start.y + CHUNK_SIZE - 1);

  for x in start.x..end.x {
    for y in (start.y..end.y).rev() {
      let noise = perlin.get([x as f64 / CHUNK_SIZE as f64, y as f64 / CHUNK_SIZE as f64]);
      let sprite_index = match noise {
        n if n > 0.7 => FOREST_TILE,
        n if n > 0.5 => GRASS_TILE,
        n if n > 0.3 => SAND_TILE,
        _ => WATER_TILE,
      };
      let grid_location = Point::new(x, y);
      let tile = Tile::new(grid_location.clone(), sprite_index, 0);
      debug!("{:?} => Noise: {}", &tile, noise);
      tiles.insert(tile);
    }
  }

  tiles
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
    generate_world(&mut commands, seed.0, asset_server, &mut texture_atlas_layouts);
  }
}
