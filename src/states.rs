use bevy::app::{App, Plugin, Update};
use bevy::log::*;
use bevy::prelude::{AppExtStates, EventReader, State, StateTransitionEvent, States};
use bevy::reflect::Reflect;
use std::fmt::Display;

pub struct AppStatePlugin;

impl Plugin for AppStatePlugin {
  fn build(&self, app: &mut App) {
    app
      .init_state::<AppState>()
      .register_type::<State<AppState>>()
      .init_state::<GenerationState>()
      .register_type::<State<GenerationState>>()
      .add_systems(
        Update,
        (log_app_state_transitions_system, log_generation_state_transitions_system),
      );
  }
}

fn log_app_state_transitions_system(mut app_state_events: EventReader<StateTransitionEvent<AppState>>) {
  for event in app_state_events.read() {
    info!(
      "Transitioning [{}] from [{}] to [{}]",
      AppState::name(),
      name_from(event.exited),
      name_from(event.entered)
    );
  }
}

fn log_generation_state_transitions_system(mut generation_state_events: EventReader<StateTransitionEvent<GenerationState>>) {
  for event in generation_state_events.read() {
    info!(
      "Transitioning [{}] from [{}] to [{}]",
      GenerationState::name(),
      name_from(event.exited),
      name_from(event.entered)
    );
  }
}

fn name_from<T: ToString>(state: Option<T>) -> String {
  match state {
    Some(state_name) => state_name.to_string(),
    None => "None".to_string(),
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

impl Display for AppState {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", format!("{:?}", self))
  }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States, Reflect)]
pub enum GenerationState {
  #[default]
  Idling,
  Generating,
}

#[allow(dead_code)]
impl GenerationState {
  pub fn name() -> &'static str {
    "GenerationState"
  }
}

impl Display for GenerationState {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", format!("{:?}", self))
  }
}
