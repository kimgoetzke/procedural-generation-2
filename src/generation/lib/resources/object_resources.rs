use crate::generation::lib::resources::asset_collection::AssetCollection;
use crate::generation::lib::{TerrainType, TileType};
use crate::generation::object::lib::TerrainState;
use crate::generation::resources::Climate;
use bevy::platform::collections::HashMap;
use bevy::prelude::Resource;

#[derive(Resource, Default, Debug, Clone)]
pub struct ObjectResources {
  terrain_climate_state_map: HashMap<(TerrainType, Climate), HashMap<TileType, Vec<TerrainState>>>,
  pub water: AssetCollection,
  pub shore: AssetCollection,
  pub l1_dry: AssetCollection,
  pub l1_moderate: AssetCollection,
  pub l1_humid: AssetCollection,
  pub l2_dry: AssetCollection,
  pub l2_moderate: AssetCollection,
  pub l2_humid: AssetCollection,
  pub l3_dry: AssetCollection,
  pub l3_moderate: AssetCollection,
  pub l3_humid: AssetCollection,
  pub animated: AssetCollection,
  pub trees_dry: AssetCollection,
  pub trees_moderate: AssetCollection,
  pub trees_humid: AssetCollection,
  pub buildings: AssetCollection,
}

impl ObjectResources {
  pub fn set_terrain_state_climate_map(
    &mut self,
    map: HashMap<(TerrainType, Climate), HashMap<TileType, Vec<TerrainState>>>,
  ) {
    self.terrain_climate_state_map = map;
  }

  pub fn get_terrain_state_collection(
    &self,
    enable_animated_objects: bool,
  ) -> HashMap<(TerrainType, Climate), HashMap<TileType, Vec<TerrainState>>> {
    let mut terrain_climate_state_map = self.terrain_climate_state_map.clone();
    for map in terrain_climate_state_map.values_mut() {
      for states in map.values_mut() {
        states.retain(|state| enable_animated_objects || !state.name.is_animated());
      }
    }
    terrain_climate_state_map
  }
}
