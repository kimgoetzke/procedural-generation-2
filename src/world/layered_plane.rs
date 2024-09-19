use crate::constants::CHUNK_SIZE;
use crate::world::plane::Plane;
use crate::world::terrain_type::TerrainType;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct LayeredPlane {
  pub planes: Vec<Plane>,
}

impl LayeredPlane {
  pub fn new(plane: Plane) -> Self {
    let mut layered_plane = vec![Plane::empty(CHUNK_SIZE as usize); TerrainType::length()];

    for layer in 0..layered_plane.len() {
      let mut current_layer = vec![vec![None; CHUNK_SIZE as usize]; CHUNK_SIZE as usize];

      for x in 0..plane.data.len() {
        for y in 0..plane.data[0].len() {
          if let Some(tile) = &plane.data[x][y] {
            if tile.layer == layer as i32 {
              current_layer[x][y] = Some(tile.clone());
            } else if tile.layer > layer as i32 {
              let modified_tile = tile.clone_with_new_terrain(TerrainType::from(layer));
              current_layer[x][y] = Some(modified_tile);
            }
          }
        }
      }

      layered_plane[layer] = Plane::new(current_layer);
    }

    Self { planes: layered_plane }
  }

  pub fn get(&self, terrain: TerrainType) -> &Plane {
    let layer = terrain as usize;
    &self.planes[layer]
  }
}
