use crate::coords::Point;
use crate::coords::point::InternalGrid;
use crate::generation::lib::{Chunk, TerrainType, TileType, shared};
use crate::resources::Settings;
use bevy::app::{App, Plugin};
use bevy::log::*;

pub struct PostProcessorPlugin;

impl Plugin for PostProcessorPlugin {
  fn build(&self, _app: &mut App) {}
}

pub(crate) fn process(mut chunk: Chunk, settings: &Settings) -> Chunk {
  let start_time = shared::get_time();
  for layer in 1..TerrainType::length() {
    let layer_name = TerrainType::from(layer);
    if layer < settings.general.spawn_from_layer || layer > settings.general.spawn_up_to_layer {
      trace!("Skipped processing [{:?}] layer because it's disabled", layer_name);
      continue;
    }
    clear_single_tiles_from_chunk_with_no_fill_below(layer, &mut chunk);
  }
  trace!(
    "Pre-processed chunk {} in {} ms on [{}]",
    chunk.coords.chunk_grid,
    shared::get_time() - start_time,
    shared::thread_name()
  );

  chunk
}

/// Removes tiles of [`TileType::Single`] that have no [`TileType::Fill`] tile below them because with the current tile
/// set sprites this will cause rendering issues e.g. a single [`TerrainType::Land2`] grass tile be rendered on top of
/// a single [`TerrainType::Land1`] "island" tile with water tile below it which doesn't look good. With a different
/// tile set this may not be necessary.
fn clear_single_tiles_from_chunk_with_no_fill_below(layer: usize, chunk: &mut Chunk) {
  let mut tiles_to_clear: Vec<(Point<InternalGrid>, Option<TileType>)> = Vec::new();
  let cg = chunk.coords.chunk_grid;
  if let (Some(this_plane), Some(plane_below)) = chunk.layered_plane.get_and_below_mut(layer) {
    tiles_to_clear = this_plane
      .data
      .iter_mut()
      .flatten()
      .filter_map(|tile| {
        if let Some(tile) = tile {
          if tile.tile_type == TileType::Single {
            if let Some(tile_below) = plane_below.get_tile(tile.coords.internal_grid) {
              if tile_below.tile_type != TileType::Fill {
                return Some((tile.coords.internal_grid, Some(tile_below.tile_type)));
              }
            } else if tile.terrain != TerrainType::ShallowWater {
              // TODO: Find out why the below happening - it doesn't appear to be a problem in practice though
              warn!(
                "Removed [{:?}] [{:?}] tile {:?} {:?} because it did not exist on the layer below",
                tile.terrain, tile.tile_type, tile.coords.tile_grid, tile.coords.internal_grid
              );
              return Some((tile.coords.internal_grid, None));
            }
          }
        }
        None
      })
      .collect();

    for (ig, _) in &tiles_to_clear {
      this_plane.clear_tile(ig);
    }
  }

  for (ig, _) in &tiles_to_clear {
    let (tile_type, terrain) = if let Some(tile) = chunk.layered_plane.get_tile_from_highest_layer(ig) {
      (tile.tile_type, tile.terrain)
    } else {
      panic!("Tile below tile {} on chunk {} was missing", ig, cg);
    };
    if let Some(tile) = chunk.layered_plane.flat.get_tile_mut(ig) {
      tile.update_to(tile_type, terrain);
    }
    trace!(
      "Updated tile {} on chunk {} to [{:?}] [{:?}] because a layer above it was cleared",
      ig, cg, tile_type, terrain
    );
  }
}
