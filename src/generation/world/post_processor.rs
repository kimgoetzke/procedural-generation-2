use crate::coordinates::Point;
use crate::coordinates::point::InternalGrid;
use crate::generation::model::{Chunk, TerrainType, TileType};
use crate::generation::shared;
use crate::settings::Settings;
use bevy::app::{App, Plugin};
use bevy::log::*;

pub struct PostProcessorPlugin;

impl Plugin for PostProcessorPlugin {
  fn build(&self, _app: &mut App) {}
}

pub fn process(mut chunk: Chunk, settings: &Settings) -> Chunk {
  let start_time = shared::get_time();
  for layer in (1..TerrainType::length()).rev() {
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
  let mut tiles_to_clear: Vec<Point<InternalGrid>> = Vec::new();
  let cg = chunk.coords.chunk_grid;
  if let (Some(this_plane), Some(plane_below)) = chunk.layered_plane.get_and_below_mut(layer) {
    tiles_to_clear = this_plane
      .data
      .iter()
      .flatten()
      .filter_map(|tile| {
        let tile = tile.as_ref()?;
        if tile.tile_type != TileType::Single {
          return None;
        }
        let tile_below = plane_below.get_tile(tile.coords.internal_grid).unwrap_or_else(|| {
          panic!(
            "Tile {} on layer {} in chunk {} has no tile directly below",
            tile.coords.internal_grid, layer, cg
          )
        });
        (tile_below.tile_type != TileType::Fill).then_some(tile.coords.internal_grid)
      })
      .collect();

    for ig in &tiles_to_clear {
      this_plane.clear_tile(ig);
    }
  }

  for ig in &tiles_to_clear {
    let tile = chunk
      .layered_plane
      .get_tile_from_highest_layer(ig)
      .unwrap_or_else(|| panic!("Tile below tile {} on chunk {} was missing", ig, cg));
    let (tile_type, terrain) = (tile.tile_type, tile.terrain);
    if let Some(tile) = chunk.layered_plane.flat.get_tile_mut(ig) {
      tile.update_to(tile_type, terrain);
    }
    trace!(
      "Updated tile {} on chunk {} to [{:?}] [{:?}] because a layer above it was cleared",
      ig, cg, tile_type, terrain
    );
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::constants::CHUNK_SIZE_PLUS_BUFFER;
  use crate::coordinates::point::InternalGrid;
  use crate::coordinates::{Coords, Point};
  use crate::generation::model::{Climate, DraftTile, LayeredPlane};

  fn chunk_with_single_land_tile() -> Chunk {
    let size = CHUNK_SIZE_PLUS_BUFFER as usize;
    let centre = size / 2;
    let mut draft_tiles = vec![vec![None; size]; size];
    for (x, column) in draft_tiles.iter_mut().enumerate() {
      for (y, tile) in column.iter_mut().enumerate() {
        let terrain = if x == centre && y == centre {
          TerrainType::Land1
        } else {
          TerrainType::Water
        };
        *tile = Some(DraftTile::new_test(
          Point::new_internal_grid(x as i32, y as i32),
          Point::new_tile_grid(x as i32, -(y as i32)),
          terrain,
          Climate::Moderate,
        ));
      }
    }

    Chunk {
      coords: Coords::new_for_chunk(Point::new_chunk_grid(0, 0)),
      climate: Climate::Moderate,
      layered_plane: LayeredPlane::new(draft_tiles, &Settings::default()),
    }
  }

  fn single_land_tile_internal_grid(chunk: &Chunk) -> Point<InternalGrid> {
    chunk.layered_plane.planes[TerrainType::Land1 as usize]
      .data
      .iter()
      .flatten()
      .flatten()
      .next()
      .expect("Land1 plane contains the test tile")
      .coords
      .internal_grid
  }

  #[test]
  fn process_clears_a_single_tile_with_no_fill_tile_below() {
    let chunk = chunk_with_single_land_tile();
    let internal_grid = single_land_tile_internal_grid(&chunk);
    let chunk = process(chunk, &Settings::default());
    assert!(
      chunk.layered_plane.planes[TerrainType::Land1 as usize]
        .get_tile(internal_grid)
        .is_none()
    );

    let flat_tile = chunk
      .layered_plane
      .flat
      .get_tile(internal_grid)
      .expect("Flat plane retains the highest remaining tile");
    assert_eq!(flat_tile.terrain, TerrainType::Shore);
    assert_eq!(flat_tile.tile_type, TileType::Single);
  }

  #[test]
  #[should_panic(expected = "has no tile directly below")]
  fn process_panics_when_a_tile_has_no_tile_directly_below() {
    let mut chunk = chunk_with_single_land_tile();
    let internal_grid = single_land_tile_internal_grid(&chunk);
    chunk.layered_plane.planes[TerrainType::Shore as usize].clear_tile(&internal_grid);
    process(chunk, &Settings::default());
  }
}
