use crate::generation::lib::resources::asset_collection::AssetCollection;
use crate::generation::lib::{TerrainType, TileType};
use crate::generation::object::lib::TerrainState;
use bevy::platform::collections::HashMap;
use bevy::prelude::Resource;

#[derive(Resource, Default, Debug, Clone)]
pub struct ObjectResources {
  pub terrain_state_map: HashMap<TerrainType, HashMap<TileType, Vec<TerrainState>>>,
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
  pub trees_dry: AssetCollection,
  pub trees_moderate: AssetCollection,
  pub trees_humid: AssetCollection,
}
