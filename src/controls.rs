use crate::events::{RefreshWorldEvent, ToggleDebugInfo};
use crate::resources::ShowDebugInfo;
use bevy::app::{App, Plugin};
use bevy::prelude::*;

pub struct ControlPlugin;

impl Plugin for ControlPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Update, player_controls_system);
  }
}

fn player_controls_system(
  keyboard_input: Res<ButtonInput<KeyCode>>,
  mut reset_world_event: EventWriter<RefreshWorldEvent>,
  mut toggle_tile_info: EventWriter<ToggleDebugInfo>,
  mut debug_info: ResMut<ShowDebugInfo>,
) {
  if keyboard_input.just_pressed(KeyCode::F2) {
    debug_info.is_on = !debug_info.is_on;
    info!(
      "[F2] Toggling debug info {}...",
      if debug_info.is_on { "on" } else { "off" }
    );
    toggle_tile_info.send(ToggleDebugInfo {});
  }
  if keyboard_input.just_pressed(KeyCode::F5) {
    info!("[F5] Refreshing world...");
    reset_world_event.send(RefreshWorldEvent {});
  }
}
