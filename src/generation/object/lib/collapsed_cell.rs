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
  pub fn new(tile_data: &'a TileData, cell: &Cell) -> Self {
    let object_name = cell.possible_states[0].name;
    let is_large_sprite = object_name.is_large_sprite();
    let sprite_index = cell.index;
    let possible_states_count = cell.possible_states.len();
    if sprite_index == -1 || possible_states_count > 1 || !cell.is_collapsed {
      error!(
        "Attempted to create collapsed cell from cell cg{:?} which is not fully collapsed",
        cell.cg,
      );
      info!(
        "Cell cg{:?} still has {} possible states: {:?}",
        cell.cg, possible_states_count, cell
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
        "Attempted to create object data from cell cg{:?} which is not fully collapsed",
        cell.cg,
      );
      info!(
        "Cell cg{:?} still has {} possible states: {:?}",
        cell.cg, possible_states_count, cell
      );
    }
    ObjectData {
      tile_data: tile_data.clone(),
      sprite_index,
      is_large_sprite,
      name: Some(object_name),
    }
  }

  pub fn from_collapsed_cell(collapsed_cell: &CollapsedCell) -> Self {
    ObjectData {
      tile_data: collapsed_cell.tile_data.clone(),
      sprite_index: collapsed_cell.sprite_index,
      is_large_sprite: collapsed_cell.is_large_sprite,
      name: collapsed_cell.name.clone(),
    }
  }
}
