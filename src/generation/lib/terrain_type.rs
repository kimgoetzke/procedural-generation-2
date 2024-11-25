use bevy::reflect::Reflect;
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(serde::Deserialize, Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Hash, Reflect)]
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
  pub fn length() -> usize {
    5 // Ignore TerrainType:Any
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

  pub fn new_clamped(proposed: TerrainType, max: i32, falloff: f64) -> Self {
    let max_layer: i32 = if falloff > 0.5 { max } else { TerrainType::length() as i32 };
    if proposed as i32 > max_layer {
      TerrainType::from(max_layer as usize)
    } else {
      proposed
    }
  }
}
