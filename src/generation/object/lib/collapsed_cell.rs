use crate::generation::lib::TileData;
use crate::generation::object::lib::{Cell, ObjectName};
use bevy::log::*;

#[derive(Debug, Clone)]
pub struct CollapsedCell<'a> {
  pub name: Option<ObjectName>,
  pub sprite_index: i32,
  pub is_large_sprite: bool,
  pub tile_data: &'a TileData,
}

impl<'a> CollapsedCell<'a> {
  pub fn new(tile_data: &'a TileData, cell_state: &Cell) -> Self {
    let is_large_sprite = cell_state.index >= 100;
    let sprite_index = if is_large_sprite {
      cell_state.index - 100
    } else {
      cell_state.index
    };
    let object_name = cell_state.possible_states[0].name;
    let possible_states_count = cell_state.possible_states.len();
    if sprite_index == -1 || possible_states_count > 1 {
      error!(
        "Attempted to create collapsed cell from cell cg{:?} which is not fully collapsed",
        cell_state.cg,
      );
      info!(
        "Cell cg{:?} has {} possible states: {:?}",
        cell_state.cg, possible_states_count, cell_state
      );
      info!("");
    }
    CollapsedCell {
      tile_data,
      sprite_index,
      is_large_sprite,
      name: Some(object_name),
    }
  }
}
