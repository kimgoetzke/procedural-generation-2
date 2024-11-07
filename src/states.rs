use bevy::app::{App, Plugin};
use bevy::prelude::{AppExtStates, States};
use bevy::reflect::Reflect;

pub struct AppStatePlugin;

impl Plugin for AppStatePlugin {
  fn build(&self, app: &mut App) {
    app
      .init_state::<AppState>()
      .register_type::<AppState>()
      .init_state::<GenerationState>()
      .register_type::<GenerationState>();
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

impl AppState {
  pub fn name() -> &'static str {
    "AppState"
  }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States, Reflect)]
pub enum GenerationState {
  #[default]
  Done,
  Generating,
}

impl GenerationState {
  pub fn name() -> &'static str {
    "GenerationState"
  }
}
