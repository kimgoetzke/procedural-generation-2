use bevy::reflect::Reflect;
use std::fmt;
use std::fmt::{Display, Formatter};
use strum::EnumIter;

#[derive(serde::Deserialize, Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Hash, Reflect, EnumIter)]
pub enum TerrainType {
  DeepWater,
  ShallowWater,
  Land1,
  Land2,
  Land3,
  Any,
}

impl Default for TerrainType {
  fn default() -> Self {
    TerrainType::Any
  }
}

impl Display for TerrainType {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}

impl TerrainType {
  /// The number of variants in the [`TerrainType`] enum excluding `Any`.
  pub fn length() -> usize {
    5
  }

  pub fn from(i: usize) -> Self {
    match i {
      0 => TerrainType::DeepWater,
      1 => TerrainType::ShallowWater,
      2 => TerrainType::Land1,
      3 => TerrainType::Land2,
      4 => TerrainType::Land3,
      _ => TerrainType::Any,
    }
  }

  pub fn new(proposed: TerrainType, is_biome_edge: bool) -> Self {
    let max_layer: i32 = if is_biome_edge {
      TerrainType::ShallowWater as i32
    } else {
      TerrainType::length() as i32
    };
    if proposed as i32 > max_layer {
      TerrainType::from(max_layer as usize)
    } else {
      proposed
    }
  }
}
