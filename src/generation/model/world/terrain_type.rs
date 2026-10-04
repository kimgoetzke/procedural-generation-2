use bevy::reflect::Reflect;
use std::fmt;
use std::fmt::{Display, Formatter};
use strum::EnumIter;

#[derive(serde::Deserialize, Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash, Reflect, EnumIter, Default)]
pub enum TerrainType {
  Water,
  Shore,
  Land1,
  Land2,
  Land3,
  #[default]
  Any,
}

impl Display for TerrainType {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}

impl TerrainType {
  /// The number of variants in the [`TerrainType`] enum excluding `Any`.
  pub const fn length() -> usize {
    5
  }

  pub const fn from(i: usize) -> Self {
    match i {
      0 => Self::Water,
      1 => Self::Shore,
      2 => Self::Land1,
      3 => Self::Land2,
      4 => Self::Land3,
      _ => Self::Any,
    }
  }

  pub const fn new(proposed: Self, is_biome_edge: bool) -> Self {
    let max_layer: i32 = if is_biome_edge {
      Self::Shore as i32
    } else {
      Self::length() as i32
    };
    if proposed as i32 > max_layer {
      Self::from(max_layer as usize)
    } else {
      proposed
    }
  }
}
