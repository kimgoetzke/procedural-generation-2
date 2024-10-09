use crate::constants::TILE_SIZE;
use crate::generation::get_time;
use crate::generation::lib::{Chunk, ObjectComponent, TerrainType, Tile, TileData, TileType};
use crate::generation::resources::AssetPacks;
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

pub fn generate(
  commands: &mut Commands,
  spawn_data: &mut Vec<(Chunk, Vec<TileData>)>,
  asset_packs: &Res<AssetPacks>,
  settings: &Res<Settings>,
) {
  if !settings.object.generate_objects {
    debug!("Skipped object generation because it's disabled");
    return;
  }
  let start_time = get_time();
  for (_, tile_data) in spawn_data.iter_mut() {
    place_trees(commands, tile_data, asset_packs, settings);
  }
  debug!("Generated objects for chunk(s) in {} ms", get_time() - start_time);
}

fn place_trees(
  commands: &mut Commands,
  tile_data: &mut Vec<TileData>,
  asset_packs: &Res<AssetPacks>,
  settings: &Res<Settings>,
) {
  let mut rng = StdRng::seed_from_u64(settings.world.noise_seed as u64);
  let forest_tiles: Vec<_> = tile_data
    .iter_mut()
    .filter_map(|t| {
      if t.tile.terrain == TerrainType::Forest && t.tile.tile_type == TileType::Fill {
        return Some(t);
      } else {
        None
      }
    })
    .collect();

  for forest_tile_data in forest_tiles {
    if rng.gen_bool(settings.object.tree_density) {
      let offset_x = rng.gen_range(-(TILE_SIZE as f32) / 2.0..=(TILE_SIZE as f32) / 2.0);
      let offset_y = rng.gen_range(-(TILE_SIZE as f32) / 2.0..=(TILE_SIZE as f32) / 2.0);
      let index = rng.gen_range(0..=4);
      trace!(
        "Placing tree at {:?} with offset ({}, {})",
        forest_tile_data.tile.coords.chunk_grid,
        offset_x,
        offset_y
      );
      commands.entity(forest_tile_data.entity).with_children(|parent| {
        parent.spawn(tree_sprite(&forest_tile_data.tile, offset_x, offset_y, index, asset_packs));
      });
    }
  }
}

fn tree_sprite(
  tile: &Tile,
  offset_x: f32,
  offset_y: f32,
  index: i32,
  asset_packs: &AssetPacks,
) -> (Name, SpriteBundle, TextureAtlas, ObjectComponent) {
  (
    Name::new("Tree Sprite"),
    SpriteBundle {
      texture: asset_packs.tree.texture.clone(),
      transform: Transform::from_xyz(
        offset_x,
        offset_y + 1.5 * TILE_SIZE as f32,
        200. - tile.coords.chunk_grid.y as f32,
      ),
      ..Default::default()
    },
    TextureAtlas {
      layout: asset_packs.tree.texture_atlas_layout.clone(),
      index: index as usize,
    },
    ObjectComponent {},
  )
}
