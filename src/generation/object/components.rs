use crate::generation::lib::TileData;
use crate::generation::object::lib::ObjectGrid;
use bevy::prelude::Component;

#[derive(Default, PartialEq)]
pub enum ObjectGenerationStatus {
  #[default]
  Pending,
  Done,
  Failure,
}

#[derive(Component)]
pub struct ObjectGenerationDataComponent {
  pub status: ObjectGenerationStatus,
  pub object_grid: ObjectGrid,
  pub tile_data: Vec<TileData>,
}

impl ObjectGenerationDataComponent {
  pub fn new(object_grid: ObjectGrid, tile_data: Vec<TileData>) -> Self {
    ObjectGenerationDataComponent {
      object_grid,
      tile_data,
      status: ObjectGenerationStatus::default(),
    }
  }
}
