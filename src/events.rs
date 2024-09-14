use bevy::prelude::{App, Event, Plugin};

pub struct SharedEventsPlugin;

impl Plugin for SharedEventsPlugin {
  fn build(&self, app: &mut App) {
    app.add_event::<RefreshWorldEvent>().add_event::<ToggleDebugInfo>();
  }
}

#[derive(Event)]
pub(crate) struct RefreshWorldEvent {}

#[derive(Event)]
pub(crate) struct ToggleDebugInfo {}
