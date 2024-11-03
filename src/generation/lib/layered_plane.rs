use crate::constants::CHUNK_SIZE_PLUS_BUFFER;
use crate::generation::lib::{DraftTile, Plane, TerrainType};
use crate::resources::Settings;

/// A `LayeredPlane` contains all relevant information about the `Tile`s in a `Chunk`. It contains a `Vec<Plane>` with
/// an `Plane` for each `TerrainType` and, for ease of use, it also contains the flat terrain data in a separate
/// `Plane`.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct LayeredPlane {
  pub planes: Vec<Plane>,
  pub flat: Plane,
}

impl LayeredPlane {
  /// Creates a new `LayeredPlane` from the flat terrain data of a `DraftChunk` by converting the terrain data into a
  /// `Plane` for each layer and converting the `DraftTile`s to `Tile`s which contain their `TileType`s.
  pub fn new(draft_tiles: Vec<Vec<Option<DraftTile>>>, settings: &Settings) -> Self {
    let mut final_layers = Vec::new();

    // Create a plane for each layer
    for layer in 0..TerrainType::length() {
      let mut current_layer = vec![vec![None; CHUNK_SIZE_PLUS_BUFFER as usize]; CHUNK_SIZE_PLUS_BUFFER as usize];

      // Skip water layer because water is not rendered
      if layer == 0 {
        final_layers.push(Plane::new(current_layer, Some(layer), settings));
        continue;
      }

      // Populate the layer using the draft plane and adjust terrain, if necessary - as a result,
      // each tile on a layer above the first rendered layer has a tile below it too
      for x in 0..draft_tiles[0].len() {
        for y in 0..draft_tiles.len() {
          if let Some(tile) = &draft_tiles[x][y] {
            if tile.layer == layer as i32 {
              current_layer[x][y] = Some(tile.clone());
            } else if tile.layer > layer as i32 {
              let modified_tile = tile.clone_with_modified_terrain(TerrainType::from(layer));
              current_layer[x][y] = Some(modified_tile);
            }
          }
        }
      }

      let plane = Plane::new(current_layer, Some(layer), settings);
      final_layers.push(plane);
    }

    Self {
      planes: final_layers,
      flat: Plane::new(draft_tiles, None, settings),
    }
  }

  pub fn get(&self, layer: usize) -> Option<&Plane> {
    if layer < self.planes.len() {
      Some(&self.planes[layer])
    } else {
      None
    }
  }

  /// Returns a tuple of mutable references with the `Plane` at the specified layer and the `Plane` below it.
  pub fn get_and_below_mut(&mut self, layer: usize) -> (Option<&mut Plane>, Option<&mut Plane>) {
    if layer == 0 {
      return (self.planes.get_mut(layer), None);
    }
    if layer >= self.planes.len() {
      return (None, None);
    }
    let (below, this_and_above) = self.planes.split_at_mut(layer);
    (this_and_above.get_mut(0), below.get_mut(layer - 1))
  }
}
