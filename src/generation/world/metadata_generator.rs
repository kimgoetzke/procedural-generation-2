use crate::states::AppState;
use bevy::app::{App, Plugin};
use bevy::log::debug;
use bevy::prelude::{NextState, OnEnter, ResMut};

pub struct MetadataGeneratorPlugin;

impl Plugin for MetadataGeneratorPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(OnEnter(AppState::Initialising), generate_metadata);
  }
}

fn generate_metadata(mut next_state: ResMut<NextState<AppState>>) {
  next_state.set(AppState::Running);
  debug!("Transitioning to [{:?}] state", AppState::Running);
}
