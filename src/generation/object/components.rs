use crate::generation::lib::TileData;
use crate::generation::object::lib::ObjectGrid;
use crate::ReflectComponent;
use bevy::app::{App, Plugin};
use bevy::prelude::{Component, Reflect};

pub struct ObjectGenerationComponentsPlugin;

impl Plugin for ObjectGenerationComponentsPlugin {
  fn build(&self, app: &mut App) {
    app.register_type::<ObjectGenerationDataComponent>();
  }
}

#[derive(Default, PartialEq, Reflect)]
pub enum ObjectGenerationStatus {
  #[default]
  Pending,
  Done,
  Failure,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
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
