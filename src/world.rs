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
  let world = commands
    .spawn((Name::new("World - Layer 0"), SpatialBundle::default()))
    .id();

  commands.entity(world).with_children(|parent| {
    let chunk = generate_chunk(seed, Point::new(0, 0));
    get_neighbours(chunk.clone()).iter().for_each(|neighbour| {
      let neighbour_chunk = generate_chunk(seed, neighbour.clone());
      spawn_tile(texture.clone(), texture_atlas_layout.clone(), parent, neighbour_chunk);
    });
    spawn_tile(texture, texture_atlas_layout, parent, chunk);
  });
}

fn spawn_tile(
  texture: Handle<Image>,
  texture_atlas_layout: Handle<TextureAtlasLayout>,
  parent: &mut ChildBuilder,
  chunk: Chunk,
) {
  for tile in chunk.tiles.iter() {
    parent.spawn((
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
}

fn generate_chunk(seed: u32, start: Point) -> Chunk {
  debug!("Generating chunk at {:?}", start);
  let mut tiles = HashSet::new();
  let perlin = Perlin::new(seed);
  let end = Point::new(start.x + CHUNK_SIZE - 1, start.y + CHUNK_SIZE - 1);
  for x in start.x..end.x {
    for y in (start.y..end.y).rev() {
      let noise = perlin.get([x as f64 / CHUNK_SIZE as f64, y as f64 / CHUNK_SIZE as f64]);
      let tile_type = match noise {
        n if n > 0.7 => TileType::Forest,
        n if n > 0.5 => TileType::Grass,
        n if n > 0.3 => TileType::Sand,
        _ => TileType::Water,
      };
      let sprite_index = match tile_type {
        TileType::Water => WATER_TILE,
        TileType::Sand => SAND_TILE,
        TileType::Grass => GRASS_TILE,
        TileType::Forest => FOREST_TILE,
      };
      let grid_location = Point::new(x, y);
      let tile = Tile::new(grid_location.clone(), tile_type, sprite_index, 0);
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
    generate_world(&mut commands, seed.0, asset_server, &mut texture_atlas_layouts);
  }
}
