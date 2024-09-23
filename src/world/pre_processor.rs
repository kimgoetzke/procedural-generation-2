use crate::resources::Settings;
use crate::world::chunk::Chunk;
use crate::world::get_time;
use crate::world::terrain_type::TerrainType;
use crate::world::tile_type::TileType;
use bevy::app::{App, Plugin};
use bevy::log::{debug, warn};
use bevy::prelude::Res;

pub struct PreProcessorPlugin;

impl Plugin for PreProcessorPlugin {
  fn build(&self, _app: &mut App) {}
}

pub fn process(mut final_chunks: Vec<Chunk>, settings: &Res<Settings>) -> Vec<Chunk> {
  let start_time = get_time();
  clear_single_tiles_with_no_fill_below(&mut final_chunks, settings);
  debug!("Pre-processed chunk(s) in {} ms", get_time() - start_time);

  final_chunks
}

fn clear_single_tiles_with_no_fill_below(final_chunks: &mut Vec<Chunk>, settings: &Res<Settings>) {
  for layer in 1..TerrainType::length() {
    let layer_name = TerrainType::from(layer);
    if layer > settings.general.spawn_up_to_layer {
      debug!("Skipped processing [{:?}] layer because it's disabled", layer_name);
      continue;
    }
    for chunk in final_chunks.iter_mut() {
      if let (Some(this_plane), Some(plane_below)) = chunk.layered_plane.get_and_below_mut(layer) {
        let tiles_to_clear: Vec<_> = this_plane
          .data
          .iter_mut()
          .flatten()
          .filter_map(|tile| {
            if let Some(tile) = tile {
              if tile.tile_type == TileType::Single {
                if let Some(tile_below) = plane_below.get_tile(tile.coords.chunk_grid) {
                  if tile_below.tile_type != TileType::Fill {
                    return Some(tile.coords.chunk_grid);
                  }
                } else {
                  warn!(
                    "{:?} tile wg{:?} removed because the layer below it was missing",
                    tile.terrain, tile.coords.world_grid
                  );
                  return Some(tile.coords.chunk_grid);
                }
              }
            }
            None
          })
          .collect();

        for coords in tiles_to_clear {
          this_plane.clear_tile(coords);
        }
      }
    }
  }
}
