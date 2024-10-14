use crate::constants::{STONES_COLUMNS, TILE_SIZE, TREES_COLUMNS};
use crate::generation::get_time;
use crate::generation::lib::{Chunk, ObjectComponent, TerrainType, Tile, TileData, TileType};
use crate::generation::resources::{AssetPacks, AssetPacksCollection};
use crate::resources::Settings;
use bevy::app::{App, Plugin};
use bevy::core::Name;
use bevy::hierarchy::BuildChildren;
use bevy::log::{debug, trace};
use bevy::prelude::{Commands, Res, SpriteBundle, TextureAtlas, Transform};
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};

pub struct ObjectGeneratorPlugin;

impl Plugin for ObjectGeneratorPlugin {
  fn build(&self, _app: &mut App) {}
}

// TODO: Generate objects asynchronously
pub fn generate(
  commands: &mut Commands,
  spawn_data: &mut Vec<(Chunk, Vec<TileData>)>,
  asset_collection: &Res<AssetPacksCollection>,
  settings: &Res<Settings>,
) {
  if !settings.object.generate_objects {
    debug!("Skipped object generation because it's disabled");
    return;
  }
  let start_time = get_time();
  for (_, tile_data) in spawn_data.iter_mut() {
    place_trees(commands, tile_data, asset_collection, settings);
    place_stones(commands, tile_data, asset_collection, settings);
  }
  debug!("Generated objects for chunk(s) in {} ms", get_time() - start_time);
}

fn place_trees(
  commands: &mut Commands,
  tile_data: &mut Vec<TileData>,
  asset_collection: &Res<AssetPacksCollection>,
  settings: &Res<Settings>,
) {
  let mut rng = StdRng::seed_from_u64(settings.world.noise_seed as u64);
  generate_objects(
    commands,
    tile_data,
    &asset_collection.trees,
    TerrainType::Forest,
    settings.object.tree_density,
    "Tree Sprite",
    TREES_COLUMNS as usize,
    &mut rng,
  );
}

fn place_stones(
  commands: &mut Commands,
  tile_data: &mut Vec<TileData>,
  asset_collection: &Res<AssetPacksCollection>,
  settings: &Res<Settings>,
) {
  let mut rng = StdRng::seed_from_u64(settings.world.noise_seed as u64);
  generate_objects(
    commands,
    tile_data,
    &asset_collection.stones,
    TerrainType::Sand,
    settings.object.stones_density,
    "Stone Sprite",
    STONES_COLUMNS as usize,
    &mut rng,
  );
}

fn generate_objects(
  commands: &mut Commands,
  tile_data: &mut Vec<TileData>,
  asset_packs: &AssetPacks,
  terrain_type: TerrainType,
  density: f64,
  sprite_name: &str,
  columns: usize,
  rng: &mut StdRng,
) {
  let relevant_tiles: Vec<_> = tile_data
    .iter_mut()
    .filter_map(|t| {
      if t.tile.terrain == terrain_type && t.tile.tile_type == TileType::Fill {
        Some(t)
      } else {
        None
      }
    })
    .collect();

  for tile_data in relevant_tiles {
    if rng.gen_bool(density) {
      let offset_x = rng.gen_range(-(TILE_SIZE as f32) / 3.0..=(TILE_SIZE as f32) / 3.0);
      let offset_y = rng.gen_range(-(TILE_SIZE as f32) / 3.0..=(TILE_SIZE as f32) / 3.0)
        + if terrain_type == TerrainType::Forest {
          TILE_SIZE as f32
        } else {
          0.0
        };
      let index = rng.gen_range(0..columns as i32);
      trace!(
        "Placing [{}] at {:?} with offset ({}, {})",
        sprite_name,
        tile_data.tile.coords.chunk_grid,
        offset_x,
        offset_y
      );
      commands.entity(tile_data.entity).with_children(|parent| {
        parent.spawn(sprite(
          &tile_data.tile,
          offset_x,
          offset_y,
          index,
          asset_packs,
          Name::new(sprite_name.to_string()),
        ));
      });
    }
  }
}

fn sprite(
  tile: &Tile,
  offset_x: f32,
  offset_y: f32,
  index: i32,
  asset_packs: &AssetPacks,
  name: Name,
) -> (Name, SpriteBundle, TextureAtlas, ObjectComponent) {
  (
    name,
    SpriteBundle {
      texture: asset_packs.stat.texture.clone(),
      transform: Transform::from_xyz(
        offset_x + TILE_SIZE as f32 / 2.0,
        offset_y,
        // TODO: Incorporate the chunk itself in the z-axis as it any chunk will render on top of the chunk below it
        200. + tile.coords.chunk_grid.y as f32,
      ),
      ..Default::default()
    },
    TextureAtlas {
      layout: asset_packs.stat.texture_atlas_layout.clone(),
      index: index as usize,
    },
    ObjectComponent {},
  )
}
