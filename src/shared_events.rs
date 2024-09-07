use bevy::prelude::{App, Event, Plugin};

pub struct SharedEventsPlugin;

impl Plugin for SharedEventsPlugin {
  fn build(&self, app: &mut App) {
    app.add_event::<RefreshWorldEvent>();
  }
}

#[derive(Event)]
pub(crate) struct RefreshWorldEvent {}
