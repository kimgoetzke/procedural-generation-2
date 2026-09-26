use crate::generation::model::Tile;
use crate::generation::object::model::{Cell, ObjectName};
use bevy::log::{error, info};
use bevy::prelude::{Entity, Reflect};

/// Contains the parent chunk entity and tile of the highest non-empty layer.
#[derive(Clone, Copy, Debug, Reflect)]
pub struct TileData {
  pub chunk_entity: Entity,
  pub flat_tile: Tile,
}

impl TileData {
  pub const fn new(parent_entity: Entity, tile: Tile) -> Self {
    Self {
      chunk_entity: parent_entity,
      flat_tile: tile,
    }
  }
}

/// Contains the data needed to spawn an object sprite.
#[derive(Debug, Clone)]
pub struct ObjectData {
  pub name: Option<ObjectName>,
  pub sprite_index: i32,
  pub is_large_sprite: bool,
  pub tile_data: TileData,
}

impl ObjectData {
  pub fn from(cell: &Cell, tile_data: &TileData) -> Self {
    let object_name = cell.get_possible_states()[0].name;
    let is_large_sprite = object_name.is_multi_tile();
    let sprite_index = cell.get_index();
    let possible_states_count = cell.get_possible_states().len();
    if sprite_index == -1 || possible_states_count > 1 || !cell.is_collapsed() {
      error!(
        "Attempted to create object data from cell {:?} which is not fully collapsed",
        cell.ig,
      );
      info!(
        "Cell {:?} still has {} possible states: {:?}",
        cell.ig, possible_states_count, cell
      );
    }

    Self {
      tile_data: *tile_data,
      sprite_index,
      is_large_sprite,
      name: Some(object_name),
    }
  }
}
