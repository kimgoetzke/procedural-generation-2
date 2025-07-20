use crate::generation::lib::TileType;
use crate::generation::lib::resources::asset_pack::AssetPack;
use bevy::platform::collections::HashSet;

#[derive(Default, Debug, Clone)]
pub struct AssetCollection {
  pub stat: AssetPack,
  pub anim: Option<AssetPack>,
  pub animated_tile_types: HashSet<TileType>,
}

impl AssetCollection {
  pub fn index_offset(&self) -> usize {
    self.stat.index_offset
  }
}
