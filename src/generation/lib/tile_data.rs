use crate::generation::lib::Tile;
use bevy::prelude::Entity;

#[derive(Clone, Copy)]
pub struct TileData {
  pub entity: Entity,
  pub parent_entity: Entity,
  pub tile: Tile,
}

impl TileData {
  pub fn new(entity: Entity, parent_entity: Entity, tile: Tile) -> Self {
    Self {
      entity,
      parent_entity,
      tile,
    }
  }
}
