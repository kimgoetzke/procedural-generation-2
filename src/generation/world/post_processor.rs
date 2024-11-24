use crate::coords::point::InternalGrid;
use crate::coords::Point;
use crate::generation::lib::{shared, Chunk, TerrainType, TileType};
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

/// Removing tiles with tile type `Single` that have no `Fill` tile below them because it will cause rendering issues
/// e.g. a single grass tile may overlap with a water tile below it which doesn't look good.
fn clear_single_tiles_from_chunk_with_no_fill_below(layer: usize, chunk: &mut Chunk) {
  let mut tiles_to_clear: Vec<(Point<InternalGrid>, Option<TileType>)> = Vec::new();
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
            } else if tile.terrain != TerrainType::Shore {
              // TODO: Find out if this is still happening at all and, if so, why it's happening
              warn!(
                "{:?} tile {:?} {:?} removed because the layer below it was missing: {:?}",
                tile.terrain, tile.coords.tile_grid, tile.coords.internal_grid, tile
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

  for (ig, tile_type) in &tiles_to_clear {
    let tile_type = if tile_type.is_none() {
      warn!(
        "Tile below tile {} was missing, assuming tile type after lowering terrain should be [Fill]",
        ig
      );
      TileType::Fill
    } else {
      tile_type.unwrap()
    };
    if let Some(tile) = chunk.layered_plane.flat.get_tile_mut(ig) {
      tile.lower_terrain_by_one(tile_type);
    }
  }
}
