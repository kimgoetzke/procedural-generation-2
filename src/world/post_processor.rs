use crate::constants::TILE_SIZE;
use crate::resources::Settings;
use crate::world::chunk::Chunk;
use crate::world::get_time;
use crate::world::resources::AssetPacks;
use crate::world::terrain_type::TerrainType;
use crate::world::tile::Tile;
use crate::world::tile_type::TileType;
use bevy::app::{App, Plugin};
use bevy::core::Name;
use bevy::log::*;
use bevy::prelude::{Commands, Res, SpriteBundle, TextureAtlas, Transform};

pub struct PostProcessorPlugin;

impl Plugin for PostProcessorPlugin {
  fn build(&self, _app: &mut App) {}
}

pub fn process(
  commands: &mut Commands,
  mut final_chunks: Vec<Chunk>,
  asset_packs: &Res<AssetPacks>,
  settings: &Res<Settings>,
) -> Vec<Chunk> {
  if !settings.general.layer_post_processing {
    debug!("Skipped post-processing because it's disabled");
    return final_chunks;
  }
  let start_time = get_time();
  place_trees(commands, &mut final_chunks, asset_packs, settings);
  debug!("Post-processed chunk(s) in {} ms", get_time() - start_time);

  final_chunks
}

fn place_trees(
  commands: &mut Commands,
  final_chunks: &mut Vec<Chunk>,
  asset_packs: &Res<AssetPacks>,
  _settings: &Res<Settings>,
) {
  for chunk in final_chunks.iter_mut() {
    if let Some(grass_plane) = chunk.layered_plane.get_by_terrain(TerrainType::Forest) {
      let candidate_tiles: Vec<_> = grass_plane
        .data
        .iter()
        .flatten()
        .filter_map(|tile| {
          if let Some(tile) = tile {
            if tile.tile_type == TileType::Fill {
              return Some(tile);
            }
          }
          None
        })
        .collect();

      for tile in candidate_tiles {
        // TODO: Spawn trees by chance based on a seed
        // TODO: Add random offset to tree placement for variety
        debug!("Placing tree at {:?} for {:?}", tile.coords.chunk_grid, tile);
        commands.spawn(tree_sprite(tile, asset_packs));
      }
    }
  }
}

fn tree_sprite(tile: &Tile, asset_packs: &AssetPacks) -> (Name, SpriteBundle, TextureAtlas) {
  (
    Name::new("Tree Sprite"),
    SpriteBundle {
      texture: asset_packs.tree.texture.clone(),
      transform: Transform::from_xyz(
        tile.coords.world.x as f32,
        tile.coords.world.y as f32 + TILE_SIZE as f32,
        200. - tile.coords.chunk_grid.y as f32,
      ),
      ..Default::default()
    },
    TextureAtlas {
      layout: asset_packs.tree.texture_atlas_layout.clone(),
      index: 0,
    },
  )
}
