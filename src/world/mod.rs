use crate::constants::*;
use crate::events::{RefreshWorldEvent, ToggleDebugInfo};
use crate::resources::{Settings, ShowDebugInfo};
use crate::world::asset_packs::{get_asset_packs, AssetPack, AssetPacks};
use crate::world::chunk::{get_chunk_spawn_points, Chunk, DraftChunk};
use crate::world::coords::Point;
use crate::world::terrain_type::TerrainType;
use crate::world::tile::DraftTile;
use crate::world::tile_type::*;
use bevy::app::{App, Plugin, Startup};
use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy_inspector_egui::egui::TextBuffer;
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

struct TileData {
  entity: Entity,
  tile: Tile,
}

impl TileData {
  fn new(entity: Entity, tile: Tile) -> Self {
    Self { entity, tile }
  }
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
  let t1 = get_time();
  let asset_packs = get_asset_packs(&asset_server, texture_atlas_layouts);

  // Generate draft chunks
  let mut draft_chunks: Vec<DraftChunk> = Vec::new();
  let spawn_point = Point::new(-(CHUNK_SIZE / 2), -(CHUNK_SIZE / 2));
  get_chunk_spawn_points(&spawn_point, CHUNK_SIZE)
    .iter()
    .for_each(|point| {
      if settings.general.generate_neighbour_chunks {
        let chunk_layer_data = generate_chunk_layer_data(point.clone(), settings);
        draft_chunks.push(chunk_layer_data);
      } else {
        if point.x == spawn_point.x && point.y == spawn_point.y {
          debug!("Skipped generating neighbour chunks because it's disabled");
          let draft_chunk = generate_chunk_layer_data(point.clone(), settings);
          draft_chunks.push(draft_chunk);
        }
      }
    });
  debug!("Generated draft chunk(s)");

  // Convert draft chunks to chunks
  let mut final_chunks: Vec<Chunk> = Vec::new();
  for draft in draft_chunks {
    let chunk = draft.to_chunk(settings);
    final_chunks.push(chunk);
  }
  debug!("Converted draft chunk(s) to chunk(s)");

  // Spawn world entity and base chunks
  let mut tile_data = Vec::new();
  commands
    .spawn((Name::new("World - Layer 0"), SpatialBundle::default(), WorldComponent))
    .with_children(|parent| {
      for chunk in final_chunks.iter() {
        let entry = spawn_chunk(&asset_packs, parent, &chunk, settings);
        tile_data.extend(entry);
      }
    });
  debug!("Spawned world and chunk entities");

  // Spawn each layer of tiles
  for i in 0..TerrainType::length() {
    let t2 = get_time();
    let terrain_type = TerrainType::from(i);
    for tile in &tile_data {
      let tile_commands = commands.entity(tile.entity);
      spawn_tile(tile_commands, &tile.tile, &asset_packs, &settings, terrain_type);
    }
    debug!("Spawned all [{:?}] tiles within {} ms", terrain_type, get_time() - t2);
  }

  info!("✅  World generation took {} ms", get_time() - t1);
}

fn spawn_chunk(
  asset_packs: &AssetPacks,
  world_child_builder: &mut ChildBuilder,
  chunk: &Chunk,
  settings: &Res<Settings>,
) -> Vec<TileData> {
  let mut tile_data = Vec::new();
  world_child_builder
    .spawn((
      Name::new(format!("Chunk ({},{})", chunk.coords.world.x, chunk.coords.world.y)),
      SpatialBundle::default(),
    ))
    .with_children(|parent| {
      for tile in chunk.tiles.iter() {
        let tile_entity = parent
          .spawn((
            Name::new(
              "Tile (".to_string() + &tile.coords.grid.x.to_string() + "," + &tile.coords.grid.y.to_string() + ")",
            ),
            SpatialBundle {
              transform: Transform::from_xyz(tile.coords.world.x as f32, tile.coords.world.y as f32, 0.),
              ..Default::default()
            },
          ))
          .with_children(|tile_parent| {
            if !settings.general.draw_terrain_sprites {
              tile_parent.spawn(default_sprite(asset_packs, tile));
            }
            if settings.general.spawn_tile_debug_info {
              tile_parent.spawn(tile_info(asset_packs, &tile));
            }
          })
          .id();
        tile_data.push(TileData::new(tile_entity, tile.clone()));
      }
    });

  tile_data
}

fn tile_info(asset_packs: &AssetPacks, tile: &&Tile) -> (Name, Text2dBundle, TileDebugInfoComponent) {
  (
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
  )
}

fn spawn_tile(
  mut commands: EntityCommands,
  tile: &Tile,
  asset_packs: &AssetPacks,
  settings: &Res<Settings>,
  terrain: TerrainType,
) {
  commands.with_children(|parent| {
    if settings.general.draw_terrain_sprites {
      match (terrain, tile.layer) {
        (TerrainType::Water, _) => {
          parent.spawn(terrain_fill_sprite(&asset_packs.default, terrain as usize));
        }
        (TerrainType::Shore, layer) if layer > SHORE_LAYER as i32 => {
          parent.spawn(terrain_fill_sprite(&asset_packs.shore, terrain as usize));
        }
        (TerrainType::Shore, layer) if layer == SHORE_LAYER as i32 => {
          parent.spawn(terrain_sprite(tile, asset_packs));
        }
        (TerrainType::Sand, layer) if layer > SAND_LAYER as i32 => {
          parent.spawn(terrain_fill_sprite(&asset_packs.sand, terrain as usize));
        }
        (TerrainType::Sand, layer) if layer == SAND_LAYER as i32 => {
          parent.spawn(terrain_sprite(tile, asset_packs));
        }
        (TerrainType::Grass, layer) if layer > GRASS_LAYER as i32 => {
          parent.spawn(terrain_fill_sprite(&asset_packs.grass, terrain as usize));
        }
        (TerrainType::Grass, layer) if layer == GRASS_LAYER as i32 => {
          parent.spawn(terrain_sprite(tile, asset_packs));
        }
        (TerrainType::Forest, layer) if layer > FOREST_LAYER as i32 => {
          parent.spawn(terrain_fill_sprite(&asset_packs.forest, terrain as usize));
        }
        (TerrainType::Forest, layer) if layer == FOREST_LAYER as i32 => {
          parent.spawn(terrain_sprite(tile, asset_packs));
        }
        _ => {}
      }
    }
    return;
  });
}

fn default_sprite(
  asset_packs: &AssetPacks,
  tile: &Tile,
) -> (Name, SpriteBundle, TextureAtlas, DefaultSpriteTileComponent) {
  (
    Name::new("Default Sprite"),
    SpriteBundle {
      texture: asset_packs.default.texture.clone(),
      transform: Transform::from_xyz(0.0, 0.0, tile.layer as f32),
      ..Default::default()
    },
    TextureAtlas {
      layout: asset_packs.default.texture_atlas_layout.clone(),
      index: tile.default_sprite_index,
    },
    DefaultSpriteTileComponent,
  )
}

fn terrain_fill_sprite(asset_pack: &AssetPack, layer: usize) -> (Name, SpriteBundle, TextureAtlas) {
  (
    Name::new("Layer Fill ".as_str().to_owned() + layer.to_string().as_str()),
    SpriteBundle {
      texture: asset_pack.texture.clone(),
      transform: Transform::from_xyz(0.0, 0.0, layer as f32 + 1.),
      ..Default::default()
    },
    TextureAtlas {
      layout: asset_pack.texture_atlas_layout.clone(),
      index: if layer == WATER_LAYER { 0 } else { FILL },
    },
  )
}

fn terrain_sprite(
  tile: &Tile,
  asset_packs: &AssetPacks,
) -> (Name, SpriteBundle, TextureAtlas, TerrainSpriteTileComponent) {
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
      transform: Transform::from_xyz(0.0, 0.0, tile.layer as f32 + 2.),
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
    // AnimationTimer {
    //   timer: Timer::from_seconds(delay + LAYER_DELAY, TimerMode::Once),
    // },
  )
}

fn generate_chunk_layer_data(start: Point, settings: &Res<Settings>) -> DraftChunk {
  let mut noise_stats: (f64, f64, f64, f64) = (5., -5., 5., -5.);
  let time = get_time();
  let perlin = Perlin::new(settings.world.noise_seed);
  let end = Point::new(start.x + CHUNK_SIZE - 1, start.y + CHUNK_SIZE - 1);
  let center = Point::new((start.x + end.x) / 2, (start.y + end.y) / 2);
  let max_distance = (CHUNK_SIZE as f64) / 2.;
  let frequency = settings.world.noise_frequency;
  let amplitude = settings.world.noise_amplitude;
  let elevation = settings.world.elevation;
  let falloff_strength = settings.world.falloff_strength;
  let mut tiles = vec![vec![None; CHUNK_SIZE as usize]; CHUNK_SIZE as usize];
  let mut cx = 0;
  let mut cy = 0;

  for gx in start.x..=end.x {
    for gy in start.y..=end.y {
      let chunk_location = Point::new(cx, cy);
      let grid_location = Point::new(gx, gy);

      // Calculate noise value
      let noise = perlin.get([gx as f64 * frequency, gy as f64 * frequency]);
      let clamped_noise = (noise * amplitude).clamp(-1., 1.);
      let normalised_noise = (clamped_noise + 1.) / 2.;
      let normalised_noise = (normalised_noise + elevation).clamp(0., 1.);

      // Adjust noise based on distance from center using falloff map
      let distance_x = (gx - center.x).abs() as f64 / max_distance;
      let distance_y = (gy - center.y).abs() as f64 / max_distance;
      let distance_from_center = distance_x.max(distance_y);
      let falloff = (1. - distance_from_center).max(0.).powf(falloff_strength);
      let adjusted_noise = normalised_noise * falloff;

      // Determine terrain type based on noise
      let tile = match adjusted_noise {
        n if n > 0.75 => DraftTile::new(chunk_location, grid_location, TerrainType::Forest, FOREST_LAYER),
        n if n > 0.6 => DraftTile::new(chunk_location, grid_location, TerrainType::Grass, GRASS_LAYER),
        n if n > 0.45 => DraftTile::new(chunk_location, grid_location, TerrainType::Sand, SAND_LAYER),
        n if n > 0.3 => DraftTile::new(chunk_location, grid_location, TerrainType::Shore, SHORE_LAYER),
        _ => DraftTile::new(chunk_location, grid_location, TerrainType::Water, WATER_LAYER),
      };

      noise_stats.0 = noise_stats.0.min(normalised_noise);
      noise_stats.1 = noise_stats.1.max(normalised_noise);
      noise_stats.2 = noise_stats.2.min(adjusted_noise);
      noise_stats.3 = noise_stats.3.max(adjusted_noise);
      trace!("{:?} => Noise: {}", &tile, adjusted_noise);

      tiles[cx as usize][cy as usize] = Some(tile);
      cy += 1;
    }
    cx += 1;
    cy = 0;
  }
  trace!("Noise: {:.2} to {:.2}", noise_stats.0, noise_stats.1);
  trace!("Adjusted noise: {:.2} to {:.2}", noise_stats.2, noise_stats.3);
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
