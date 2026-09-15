use crate::generation::lib::{TerrainType, TileType};
use bevy::log::*;
use std::cmp::Reverse;

#[derive(Clone, Debug)]
pub struct TileBelow {
  pub(crate) terrain: TerrainType,
  pub(crate) tile_type: TileType,
  pub(crate) below: Option<Box<Self>>,
}

impl Default for TileBelow {
  fn default() -> Self {
    Self {
      terrain: TerrainType::Any,
      tile_type: TileType::Unknown,
      below: None,
    }
  }
}

impl TileBelow {
  pub fn new(mut data: Vec<(TerrainType, TileType)>) -> Self {
    data.sort_by_key(|item| Reverse(item.0));
    if data.is_empty() {
      unreachable!("You must not call TileBelow::new with an empty data vector");
    }
    let mut iter = data.into_iter();
    if let Some((terrain, tile_type)) = iter.next() {
      Self {
        terrain,
        tile_type,
        below: {
          let rest: Vec<_> = iter.collect();
          if rest.is_empty() {
            None
          } else {
            Some(Box::new(Self::new(rest)))
          }
        },
      }
    } else {
      Self::default()
    }
  }

  pub fn log(&self) {
    trace!(
      "- Tile below is [{:?}] of type [{:?}] and {}",
      self.terrain,
      self.tile_type,
      if self.below.is_some() {
        "has a tile below it"
      } else {
        "does not have a tile below"
      }
    );
    if let Some(below) = &self.below {
      below.log();
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  impl TileBelow {
    pub fn from(terrain: TerrainType, tile_type: TileType, below: Option<Box<Self>>) -> Self {
      Self {
        terrain,
        tile_type,
        below,
      }
    }
  }
}
