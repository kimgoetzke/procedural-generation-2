use crate::coords::Point;
use bevy::prelude::{App, Event, Plugin};

pub struct SharedEventsPlugin;

impl Plugin for SharedEventsPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_event::<RegenerateWorldEvent>()
      .add_event::<ToggleDebugInfo>()
      .add_event::<MouseClickEvent>()
      .add_event::<UpdateWorldEvent>()
      .add_event::<DespawnDistantChunkEvent>();
  }
}

#[derive(Event)]
pub struct RegenerateWorldEvent {}

#[derive(Event)]
pub struct ToggleDebugInfo {}

#[derive(Event)]
pub struct MouseClickEvent {
  pub world: Point,
  pub world_grid: Point,
}

#[derive(Event)]
pub struct UpdateWorldEvent {
  pub world: Point,
  pub world_grid: Point,
}

#[derive(Event)]
pub struct DespawnDistantChunkEvent {}
