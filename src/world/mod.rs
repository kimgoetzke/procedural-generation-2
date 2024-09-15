use crate::constants::*;
use crate::events::{RefreshWorldEvent, ToggleDebugInfo};
use crate::resources::{Settings, ShowDebugInfo};
use crate::world::asset_packs::{get_asset_packs, AssetPacks};
use crate::world::chunk::{get_neighbour_world_points, Chunk, DraftChunk};
use crate::world::coords::Point;
use crate::world::terrain_type::TerrainType;
use crate::world::tile::DraftTile;
use crate::world::tile_type::*;
use bevy::app::{App, Plugin, Startup};
use bevy::prelude::*;
use noise::{NoiseFn, Perlin};
use std::time::SystemTime;
use tile::Tile;

mod asset_packs;
mod chunk;
mod coords;
mod neighbours;
mod terrain_type;
mod tile;
mod tile_type;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Startup, generate_world_system).add_systems(
      Update,
      (refresh_world_event, toggle_tile_info_event, update_visibility_system),
    );
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

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer {
  timer: Timer,
}

fn generate_world_system(
  mut commands: Commands,
  asset_server: Res<AssetServer>,
  mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
  settings: Res<Settings>,
) {
  spawn_world(&mut commands, asset_server, &mut texture_atlas_layouts, &settings);
}

fn spawn_world(
  commands: &mut Commands,
  asset_server: Res<AssetServer>,
  texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
  settings: &Res<Settings>,
) {
  let timestamp = get_time();
  let asset_packs = get_asset_packs(&asset_server, texture_atlas_layouts);

  commands
    .spawn((Name::new("World - Layer 0"), SpatialBundle::default(), WorldComponent))
    .with_children(|parent| {
      // Generate data for the initial chunk
      let spawn_point = Point::new(-(CHUNK_SIZE / 2), -(CHUNK_SIZE / 2));
      let draft_chunk = generate_chunk_layer_data(spawn_point, settings);
      let mut draft_chunks: Vec<DraftChunk> = vec![draft_chunk.clone()];

      // Generate data for all neighbouring chunks
      // get_neighbour_world_points(&draft_chunk.coords, CHUNK_SIZE)
      //   .iter()
      //   .for_each(|point| {
      //     draft_chunks.push(generate_chunk_layer_data(seed, point.clone()));
      //   });

      // Spawn all chunks
      for draft in draft_chunks {
        let chunk = draft.to_chunk(settings);
        spawn_chunk(&asset_packs, parent, chunk, settings);
      }
    });

  info!("✅  World generation took {} ms", get_time() - timestamp);
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
      if !settings.draw_terrain_sprites || tile.terrain == TerrainType::Water {
        parent.spawn(default_sprite(asset_packs, tile, delay));
      }

      if settings.draw_terrain_sprites {
        // Lower layer terrain sprite
        // if tile.layer > 1 {
        //   parent.spawn((
        //     Name::new("Lower Layer Terrain Sprite"),
        //     SpriteBundle {
        //       texture: match tile.terrain {
        //         TerrainType::Sand => asset_packs.sand.texture.clone(),
        //         _ => asset_packs.default.texture.clone(),
        //       },
        //       transform: Transform::from_xyz(0.0, 0.0, tile.layer as f32 + 5.),
        //       ..Default::default()
        //     },
        //     TextureAtlas {
        //       layout: match tile.terrain {
        //         TerrainType::Sand => asset_packs.sand.texture_atlas_layout.clone(),
        //         _ => asset_packs.default.texture_atlas_layout.clone(),
        //       },
        //       index: FILL,
        //     },
        //     TerrainSpriteTileComponent,
        //     AnimationTimer {
        //       timer: Timer::from_seconds(delay + LAYER_DELAY / 2., TimerMode::Once),
        //     },
        //   ));
        // }

        // The terrain sprite
        if tile.terrain != TerrainType::Water {
          parent.spawn(terrain_sprite(asset_packs, &tile, delay));
        }
      }

      // The tile debug info
      if settings.spawn_tile_debug_info {
        parent.spawn((
          Name::new("Tile Debug Info"),
          Text2dBundle {
            text: Text::from_section(
              format!(
                "g{:?}\n{:?}\n{:?}\nSprite index {:?}\nLayer {:?}",
                tile.coords.grid,
                tile.terrain,
                tile.tile_type,
                get_sprite_index(&tile),
                tile.layer
              ),
              TextStyle {
                font: asset_packs.font.clone(),
                font_size: 22.,
                color: Color::WHITE,
              },
            )
            .with_justify(JustifyText::Center),
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
      }
    });
}

fn default_sprite(
  asset_packs: &AssetPacks,
  tile: &Tile,
  delay: f32,
) -> (
  Name,
  SpriteBundle,
  TextureAtlas,
  DefaultSpriteTileComponent,
  AnimationTimer,
) {
  (
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
  )
}

fn terrain_sprite(
  asset_packs: &AssetPacks,
  tile: &&Tile,
  delay: f32,
) -> (
  Name,
  SpriteBundle,
  TextureAtlas,
  TerrainSpriteTileComponent,
  AnimationTimer,
) {
  (
    Name::new("Terrain Sprite"),
    SpriteBundle {
      texture: match tile.terrain {
        TerrainType::Shore => asset_packs.shore.texture.clone(),
        TerrainType::Sand => asset_packs.sand.texture.clone(),
        TerrainType::Grass => asset_packs.grass.texture.clone(),
        TerrainType::Forest => asset_packs.forest.texture.clone(),
        _ => panic!("Invalid terrain type for drawing a terrain sprite"),
      },
      transform: Transform::from_xyz(0.0, 0.0, tile.layer as f32 + 10.),
      visibility: Visibility::Hidden,
      ..Default::default()
    },
    TextureAtlas {
      layout: match tile.terrain {
        TerrainType::Shore => asset_packs.shore.texture_atlas_layout.clone(),
        TerrainType::Sand => asset_packs.sand.texture_atlas_layout.clone(),
        TerrainType::Grass => asset_packs.grass.texture_atlas_layout.clone(),
        TerrainType::Forest => asset_packs.forest.texture_atlas_layout.clone(),
        _ => panic!("Invalid terrain type for drawing a terrain sprite"),
      },
      index: get_sprite_index(&tile),
    },
    TerrainSpriteTileComponent,
    AnimationTimer {
      timer: Timer::from_seconds(delay + LAYER_DELAY, TimerMode::Once),
    },
  )
}

fn generate_chunk_layer_data(start: Point, settings: &Res<Settings>) -> DraftChunk {
  let mut tiles: Vec<DraftTile> = Vec::new();
  let mut noise_stats: (f64, f64, f64, f64) = (5., -5., 5., -5.);
  let time = get_time();
  let perlin = Perlin::new(settings.world_gen.noise_seed);
  let end = Point::new(start.x + CHUNK_SIZE - 1, start.y + CHUNK_SIZE - 1);
  let center = Point::new((start.x + end.x) / 2, (start.y + end.y) / 2);
  let max_distance = (CHUNK_SIZE as f64) / 2.;
  let frequency = settings.world_gen.noise_frequency;
  let amplitude = settings.world_gen.noise_amplitude;
  let elevation = settings.world_gen.elevation;
  let falloff_strength = settings.world_gen.falloff_strength;

  for x in start.x..end.x {
    for y in (start.y..end.y).rev() {
      // Calculate noise value
      let noise = perlin.get([x as f64 * frequency, y as f64 * frequency]);
      let clamped_noise = (noise * amplitude).clamp(-1., 1.);
      let normalised_noise = (clamped_noise + 1.) / 2.;
      let normalised_noise = (normalised_noise + elevation).clamp(0., 1.);

      // Adjust noise based on distance from center using falloff map
      let distance_x = (x - center.x).abs() as f64 / max_distance;
      let distance_y = (y - center.y).abs() as f64 / max_distance;
      let distance_from_center = distance_x.max(distance_y);
      let falloff = (1. - distance_from_center).max(0.).powf(falloff_strength);
      let adjusted_noise = normalised_noise * falloff;

      // Determine terrain type based on noise
      let tile = match adjusted_noise {
        n if n > 0.75 => DraftTile::new(Point::new(x, y), TerrainType::Forest, FOREST_TILE),
        n if n > 0.6 => DraftTile::new(Point::new(x, y), TerrainType::Grass, GRASS_TILE),
        n if n > 0.45 => DraftTile::new(Point::new(x, y), TerrainType::Sand, SAND_TILE),
        n if n > 0.3 => DraftTile::new(Point::new(x, y), TerrainType::Shore, SHORE_TILE),
        _ => DraftTile::new(Point::new(x, y), TerrainType::Water, WATER_TILE),
      };

      noise_stats.0 = noise_stats.0.min(normalised_noise);
      noise_stats.1 = noise_stats.1.max(normalised_noise);
      noise_stats.2 = noise_stats.2.min(adjusted_noise);
      noise_stats.3 = noise_stats.3.max(adjusted_noise);
      trace!("{:?} => Noise: {}", &tile, noise);

      tiles.push(tile);
    }
  }
  debug!("Noise: {:.2} to {:.2}", noise_stats.0, noise_stats.1);
  debug!("Adjusted noise: {:.2} to {:.2}", noise_stats.2, noise_stats.3);
  debug!("Generated draft chunk at {:?} within {} ms", start, get_time() - time);

  DraftChunk::new(start, tiles)
}

fn get_time() -> u128 {
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
  asset_server: Res<AssetServer>,
  mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
  settings: Res<Settings>,
) {
  let event_count = events.read().count();
  if event_count > 0 {
    for world in existing_worlds.iter() {
      commands.entity(world).despawn_recursive();
    }
    spawn_world(&mut commands, asset_server, &mut texture_atlas_layouts, &settings);
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
