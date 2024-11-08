use crate::generation::lib::TileData;
use crate::generation::object::lib::{Cell, ObjectName};
use bevy::log::*;

/// Represents data associated with an object in the game world. Created as part of the object generation process and
/// fed into the code that spawns the resulting object sprites in the game world.
#[derive(Debug, Clone)]
pub struct ObjectData {
  pub name: Option<ObjectName>,
  pub sprite_index: i32,
  pub is_large_sprite: bool,
  pub tile_data: TileData,
}

impl ObjectData {
  pub fn from_wfc_cell(tile_data: &TileData, cell: &Cell) -> Self {
    let object_name = cell.possible_states[0].name;
    let is_large_sprite = object_name.is_large_sprite();
    let sprite_index = cell.index;
    let possible_states_count = cell.possible_states.len();
    if sprite_index == -1 || possible_states_count > 1 || !cell.is_collapsed {
      error!(
        "Attempted to create object data from cell {:?} which is not fully collapsed",
        cell.ig,
      );
      info!(
        "Cell {:?} still has {} possible states: {:?}",
        cell.ig, possible_states_count, cell
      );
    }
    ObjectData {
      tile_data: tile_data.clone(),
      sprite_index,
      is_large_sprite,
      name: Some(object_name),
    }
  }
}
