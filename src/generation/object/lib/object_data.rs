use crate::generation::lib::TileData;
use crate::generation::object::lib::{Cell, ObjectName};
use bevy::log::*;

#[derive(Debug, Clone)]
pub struct ObjectData {
  pub name: Option<ObjectName>,
  pub sprite_index: i32,
  pub is_large_sprite: bool,
  pub tile_data: TileData,
}

impl ObjectData {
  pub fn new(tile_data: &TileData, cell: &Cell) -> Self {
    let object_name = cell.possible_states[0].name;
    let is_large_sprite = object_name.is_large_sprite();
    let sprite_index = cell.index;
    let possible_states_count = cell.possible_states.len();
    if sprite_index == -1 || possible_states_count > 1 || !cell.is_collapsed {
      error!(
        "Attempted to create object data from cell ig{:?} which is not fully collapsed",
        cell.ig,
      );
      info!(
        "Cell ig{:?} still has {} possible states: {:?}",
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
