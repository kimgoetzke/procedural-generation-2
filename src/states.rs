use bevy::app::{App, Plugin};
use bevy::prelude::{AppExtStates, State, States};
use bevy::reflect::Reflect;

pub struct AppStatePlugin;

impl Plugin for AppStatePlugin {
  fn build(&self, app: &mut App) {
    app
      .init_state::<AppState>()
      .register_type::<State<AppState>>()
      .init_state::<GenerationState>()
      .register_type::<State<GenerationState>>();
  }
}

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

#[allow(dead_code)]
impl GenerationState {
  pub fn name() -> &'static str {
    "GenerationState"
  }
}
