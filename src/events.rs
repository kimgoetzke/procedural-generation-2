use crate::coords::Coords;
use bevy::prelude::{App, Event, Plugin};

pub struct SharedEventsPlugin;

impl Plugin for SharedEventsPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_event::<RefreshWorldEvent>()
      .add_event::<ToggleDebugInfo>()
      .add_event::<MouseClickEvent>();
  }
}

#[derive(Event)]
pub struct RefreshWorldEvent {}

#[derive(Event)]
pub struct ToggleDebugInfo {}

#[derive(Event)]
pub struct MouseClickEvent {
  pub coords: Coords,
}
