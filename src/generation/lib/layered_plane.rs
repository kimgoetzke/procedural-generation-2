use crate::constants::CHUNK_SIZE_PLUS_BUFFER;
use crate::coords::Point;
use crate::coords::point::InternalGrid;
use crate::generation::lib::{DraftTile, Plane, TerrainType, Tile};
use crate::resources::Settings;

/// A [`LayeredPlane`] contains all relevant information about the [`crate::generation::lib::Tile`]s in a
/// [`crate::generation::lib::Chunk`]. It contains a [`Vec<Plane>`] with an [`Plane`] for each [`TerrainType`] and, for
/// ease of use, it also contains the flat terrain data in a separate [`Plane`].
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct LayeredPlane {
  /// A vector of [`Plane`]s, each representing a layer of terrain data for a specific [`TerrainType`]. The first
  pub planes: Vec<Plane>,
  /// The flat plane effectively contains only the highest layer of terrain data for a given tile, which is the most
  /// visible terrain layer after rendering.
  pub flat: Plane,
}

impl LayeredPlane {
  /// Creates a new [`LayeredPlane`] from the flat terrain data of vector of draft tiles by converting the terrain data
  /// into a [`Plane`] for each layer and converting the [`DraftTile`]s to [`crate::generation::lib::Tile`]s which
  /// contain their [`crate::generation::lib::TileType`]s.
  pub fn new(draft_tiles: Vec<Vec<Option<DraftTile>>>, settings: &Settings) -> Self {
    let mut final_layers = Vec::new();

    // Create a plane for each layer
    for layer in 0..TerrainType::length() {
      let mut current_layer = vec![vec![None; CHUNK_SIZE_PLUS_BUFFER as usize]; CHUNK_SIZE_PLUS_BUFFER as usize];

      // Skip lowest water layer because water is not rendered since the background colour is used instead
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

  /// Returns a tuple of mutable references with the [`Plane`] at the specified layer and the [`Plane`] below it.
  pub fn get_and_below_mut(&mut self, layer: usize) -> (Option<&mut Plane>, Option<&mut Plane>) {
    match layer {
      0 => (self.planes.get_mut(layer), None),
      _ if layer >= self.planes.len() => (None, None),
      _ => {
        let (below, this_and_above) = self.planes.split_at_mut(layer);
        (this_and_above.get_mut(0), below.get_mut(layer - 1))
      }
    }
  }

  /// Returns a reference to the tile on the highest layer at the specified internal grid coordinates, if it exists.
  /// This function does not use the [`LayeredPlane::flat`] plane, but instead uses the max layer from the
  /// [`LayeredPlane::planes`] vector to find the tile.
  pub fn get_tile_from_highest_layer(&mut self, ig: &Point<InternalGrid>) -> Option<&Tile> {
    self
      .planes
      .iter()
      .filter(|plane| if let Some(_) = plane.get_tile(*ig) { true } else { false })
      .max_by(|a, b| a.layer.cmp(&b.layer))
      .map(|plane| plane.get_tile(*ig).expect("Tile should exist on the highest layer"))
  }
}
