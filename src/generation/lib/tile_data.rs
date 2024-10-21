use crate::generation::lib::Tile;
use bevy::prelude::Entity;
use bevy::reflect::Reflect;

/// Contains the tile entity, parent chunk entity, and tile of the highest, non-empty layer of a tile.
#[derive(Clone, Copy, Debug, Reflect)]
pub struct TileData {
  pub entity: Entity,
  pub chunk_entity: Entity,
  pub flat_tile: Tile,
}

impl TileData {
  pub fn new(entity: Entity, parent_entity: Entity, tile: Tile) -> Self {
    Self {
      entity,
      chunk_entity: parent_entity,
      flat_tile: tile,
    }
  }
}
