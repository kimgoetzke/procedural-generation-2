use crate::events::{RefreshWorldEvent, ToggleDebugInfo};
use crate::resources::{Settings, ShowDebugInfo};
use crate::settings::*;
use crate::world::chunk::{get_chunk_neighbour_points, Chunk, DraftChunk};
use crate::world::shared::*;
use crate::world::tile::DraftTile;
use bevy::app::{App, Plugin, Startup};
use bevy::prelude::*;
use noise::{NoiseFn, Perlin};
use std::time::SystemTime;
use tile::Tile;

mod chunk;
mod neighbours;
mod shared;
mod tile;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(Startup, generate_world_system)
      .add_systems(
        Update,
        (refresh_world_event, toggle_tile_info_event, update_visibility_system),
      )
      .insert_resource(Seed(1));
  }
}

#[derive(Component)]
struct WorldComponent;

#[derive(Component)]
struct DefaultSpriteTileComponent;

#[derive(Component)]
struct TerrainSpriteTileComponent;

#[derive(Component)]
struct TileDebugInfoComponent;

#[derive(Resource)]
struct Seed(u32);

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer {
  timer: Timer,
}

fn generate_world_system(
  mut commands: Commands,
  seed: Res<Seed>,
  asset_server: Res<AssetServer>,
  mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
  settings: Res<Settings>,
) {
  spawn_world(
    &mut commands,
    seed.0,
    asset_server,
    &mut texture_atlas_layouts,
    &settings,
  );
}

fn spawn_world(
  commands: &mut Commands,
  seed: u32,
  asset_server: Res<AssetServer>,
  texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
  settings: &Res<Settings>,
) {
  let timestamp = get_timestamp();
  let asset_packs = get_asset_packs(&asset_server, texture_atlas_layouts);

  commands
    .spawn((Name::new("World - Layer 0"), SpatialBundle::default(), WorldComponent))
    .with_children(|parent| {
      // Generate data for the initial chunk
      let draft_chunk = generate_chunk_layer_data(seed, Point::new(-(CHUNK_SIZE / 2), -(CHUNK_SIZE / 2)));
      let mut draft_chunks: Vec<DraftChunk> = vec![draft_chunk.clone()];

      // Generate data for all neighbouring chunks
      get_chunk_neighbour_points(&draft_chunk.coords)
        .iter()
        .for_each(|point| {
          draft_chunks.push(generate_chunk_layer_data(seed, point.clone()));
        });

      // Spawn all chunks
      for draft in draft_chunks {
        let chunk = draft.to_chunk(settings);
        spawn_chunk(&asset_packs, parent, chunk, settings);
      }
    });

  info!("✅  World generation took {} ms", get_timestamp() - timestamp);
}

fn spawn_chunk(
  asset_packs: &AssetPacks,
  world_child_builder: &mut ChildBuilder,
  chunk: Chunk,
  settings: &Res<Settings>,
) {
  world_child_builder
    .spawn((
      Name::new(format!("Chunk ({},{})", chunk.coords.world.x, chunk.coords.world.y)),
      SpatialBundle::default(),
    ))
    .with_children(|parent| {
      let mut visibility_delay = 0.;
      for tile in chunk.tiles.iter() {
        spawn_tile(asset_packs, parent, &tile, visibility_delay, settings);
        visibility_delay += BASE_DELAY;
      }
    });
}

fn spawn_tile(
  asset_packs: &AssetPacks,
  chunk_child_builder: &mut ChildBuilder,
  tile: &Tile,
  delay: f32,
  settings: &Res<Settings>,
) {
  chunk_child_builder
    .spawn((
      Name::new("Tile (".to_string() + &tile.coords.grid.x.to_string() + "," + &tile.coords.grid.y.to_string() + ")"),
      SpatialBundle {
        transform: Transform::from_xyz(tile.coords.world.x as f32, tile.coords.world.y as f32, 0.),
        ..Default::default()
      },
    ))
    .with_children(|parent| {
      // The default sprite as a base colour
      parent.spawn((
        Name::new("Default Sprite"),
        SpriteBundle {
          texture: asset_packs.default.texture.clone(),
          transform: Transform::from_xyz(0.0, 0.0, tile.layer as f32),
          visibility: Visibility::Hidden,
          ..Default::default()
        },
        TextureAtlas {
          layout: asset_packs.default.texture_atlas_layout.clone(),
          index: tile.default_sprite_index,
        },
        DefaultSpriteTileComponent,
        AnimationTimer {
          timer: Timer::from_seconds(delay, TimerMode::Once),
        },
      ));

      // The terrain sprite
      if settings.draw_terrain_sprites {
        parent.spawn((
          Name::new("Terrain Sprite"),
          SpriteBundle {
            texture: match tile.terrain {
              TerrainType::Sand => asset_packs.sand.texture.clone(),
              _ => asset_packs.default.texture.clone(),
            },
            transform: Transform::from_xyz(0.0, 0.0, tile.layer as f32 + 10.),
            visibility: Visibility::Hidden,
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
          TerrainSpriteTileComponent,
          AnimationTimer {
            timer: Timer::from_seconds(delay + LAYER_DELAY, TimerMode::Once),
          },
        ));
      }

      // The tile debug info
      let text = format!(
        "{:?}\n{:?}\n{:?}\nSprite index {:?}\nBase layer {:?}",
        tile.coords.grid,
        tile.terrain,
        tile.tile_type,
        get_sprite_index(&tile),
        tile.layer
      );
      parent.spawn((
        Name::new("Tile Debug Info"),
        Text2dBundle {
          text: Text::from_section(
            text,
            TextStyle {
              font: Default::default(),
              font_size: 22.,
              color: Color::WHITE,
            },
          )
          .with_justify(JustifyText::Left),
          visibility: Visibility::Hidden,
          transform: Transform {
            scale: Vec3::splat(0.1),
            translation: Vec3::new(0.0, 0.0, tile.layer as f32 + 20.),
            ..Default::default()
          },
          ..default()
        },
        TileDebugInfoComponent,
      ));
    });
}

fn generate_chunk_layer_data(seed: u32, start: Point) -> DraftChunk {
  let mut tiles: Vec<DraftTile> = Vec::new();
  let mut noise_stats: (f64, f64) = (0., 0.);
  let timestamp = get_timestamp();
  let perlin = Perlin::new(seed);
  let end = Point::new(start.x + CHUNK_SIZE - 1, start.y + CHUNK_SIZE - 1);

  for x in start.x..end.x {
    for y in (start.y..end.y).rev() {
      let noise = perlin.get([x as f64 / CHUNK_SIZE as f64, y as f64 / CHUNK_SIZE as f64]);
      let tile = match noise {
        n if n > 0.9 => DraftTile::new(Point::new(x, y), TerrainType::Forest, FOREST_TILE),
        n if n > 0.6 => DraftTile::new(Point::new(x, y), TerrainType::Grass, GRASS_TILE),
        n if n > 0.4 => DraftTile::new(Point::new(x, y), TerrainType::Sand, SAND_TILE),
        n if n > 0.1 => DraftTile::new(Point::new(x, y), TerrainType::Shore, SHORE_TILE),
        _ => DraftTile::new(Point::new(x, y), TerrainType::Water, WATER_TILE),
      };
      // let tile = match adjusted_noise {
      //   n if n > 0.6 => DraftTile::new(Point::new(x, y), TerrainType::Sand, SAND_TILE),
      //   _ => DraftTile::new(Point::new(x, y), TerrainType::Water, WATER_TILE),
      // };

      noise_stats.0 = noise_stats.0.min(noise);
      noise_stats.1 = noise_stats.1.max(noise);

      trace!("{:?} => Noise: {}", &tile, noise);
      tiles.push(tile);
    }
  }
  debug!("Noise: {:.2} to {:.2}", noise_stats.0, noise_stats.1,);
  debug!(
    "✅  Generated chunk layer data for chunk at {:?} within {} ms",
    start,
    get_timestamp() - timestamp
  );

  DraftChunk::new(start, tiles)
}

fn get_timestamp() -> u128 {
  SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_millis()
}

fn update_visibility_system(time: Res<Time>, mut query: Query<(&mut Visibility, &mut AnimationTimer)>) {
  for (mut visibility, mut animation) in query.iter_mut() {
    animation.timer.tick(time.delta());
    if animation.timer.finished() {
      *visibility = Visibility::Visible;
    }
  }
}

fn refresh_world_event(
  mut commands: Commands,
  mut events: EventReader<RefreshWorldEvent>,
  existing_worlds: Query<Entity, With<WorldComponent>>,
  mut seed: ResMut<Seed>,
  asset_server: Res<AssetServer>,
  mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
  settings: Res<Settings>,
) {
  let event_count = events.read().count();
  if event_count > 0 {
    for world in existing_worlds.iter() {
      commands.entity(world).despawn_recursive();
    }
    seed.0 += 1;
    spawn_world(
      &mut commands,
      seed.0,
      asset_server,
      &mut texture_atlas_layouts,
      &settings,
    );
  }
}

fn toggle_tile_info_event(
  mut events: EventReader<ToggleDebugInfo>,
  mut query: Query<&mut Visibility, With<TileDebugInfoComponent>>,
  debug_info: Res<ShowDebugInfo>,
) {
  let event_count = events.read().count();
  if event_count > 0 {
    for mut visibility in query.iter_mut() {
      *visibility = if debug_info.is_on {
        Visibility::Visible
      } else {
        Visibility::Hidden
      };
    }
  }
}
