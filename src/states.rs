use bevy::app::{App, Plugin};
use bevy::prelude::{AppExtStates, States};
use bevy::reflect::Reflect;

pub struct AppStatePlugin;

impl Plugin for AppStatePlugin {
  fn build(&self, app: &mut App) {
    app.init_state::<AppState>().register_type::<AppState>();
  }
}

// TODO: Display state in Bevy Inspector UI
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States, Reflect)]
pub enum AppState {
  #[default]
  Loading,
  Initialising,
  Running,
}
