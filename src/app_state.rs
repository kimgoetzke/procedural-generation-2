use bevy::app::{App, Plugin};
use bevy::prelude::{AppExtStates, States};

pub struct AppStatePlugin;

impl Plugin for AppStatePlugin {
  fn build(&self, app: &mut App) {
    app.init_state::<AppState>();
  }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
  #[default]
  Loading,
  Generating,
  Running,
}
