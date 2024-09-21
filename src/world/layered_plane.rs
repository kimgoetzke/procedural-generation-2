use crate::constants::CHUNK_SIZE;
use crate::resources::Settings;
use crate::world::plane::Plane;
use crate::world::terrain_type::TerrainType;
use crate::world::tile::DraftTile;
use bevy::prelude::Res;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct LayeredPlane {
  pub planes: Vec<Plane>,
  pub flat: Plane,
}

impl LayeredPlane {
  pub fn new(draft_tiles: Vec<Vec<Option<DraftTile>>>, settings: &Res<Settings>) -> Self {
    let mut final_layers = Vec::new();

    // Create a plane for each layer, except water because water is not rendered
    for layer in 1..TerrainType::length() {
      let mut current_layer = vec![vec![None; CHUNK_SIZE as usize]; CHUNK_SIZE as usize];

      // Populate the layer using the draft plane and adjust terrain if necessary
      for x in 0..draft_tiles.len() {
        for y in 0..draft_tiles[0].len() {
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

  #[allow(dead_code)]
  pub fn get(&self, terrain: TerrainType) -> &Plane {
    let layer = terrain as usize;
    &self.planes[layer]
  }
}
