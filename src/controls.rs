use crate::events::{RefreshWorldEvent, ToggleDebugInfo};
use crate::resources::{Settings, ShowDebugInfo};
use bevy::app::{App, Plugin};
use bevy::prelude::*;

pub struct ControlPlugin;

impl Plugin for ControlPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Update, (non_settings_controls_system, settings_controls_system));
  }
}

/**
 * A system that handles non-settings related controls. Note that the visibility of the settings UI is handled by the UI
 * plugin directly.
 */
fn non_settings_controls_system(
  keyboard_input: Res<ButtonInput<KeyCode>>,
  mut reset_world_event: EventWriter<RefreshWorldEvent>,
  mut toggle_tile_info: EventWriter<ToggleDebugInfo>,
  mut debug_info: ResMut<ShowDebugInfo>,
) {
  // Toggle debug info
  if keyboard_input.just_pressed(KeyCode::F3) {
    debug_info.is_on = !debug_info.is_on;
    info!(
      "[F3] Toggling debug info [{}]...",
      if debug_info.is_on { "on" } else { "off" }
    );
    toggle_tile_info.send(ToggleDebugInfo {});
  }

  // Refresh world
  if keyboard_input.just_pressed(KeyCode::F5) {
    info!("[F5] Refreshing world...");
    reset_world_event.send(RefreshWorldEvent {});
  }
}

/**
 * A system that handles setting value changes.
 */
fn settings_controls_system(keyboard_input: Res<ButtonInput<KeyCode>>, mut settings: ResMut<Settings>) {
  // Seed
  if keyboard_input.just_pressed(KeyCode::KeyR) {
    settings.world_gen.noise_seed = settings.world_gen.noise_seed.saturating_add(1);
    info!("[R] Increased noise seed to [{}]", settings.world_gen.noise_seed);
  } else if keyboard_input.just_pressed(KeyCode::KeyF) {
    settings.world_gen.noise_seed = settings.world_gen.noise_seed.saturating_sub(1);
    info!("[F] Decreased noise seed to [{}]", settings.world_gen.noise_seed);
  }

  // Frequency
  if keyboard_input.just_pressed(KeyCode::KeyT) {
    settings.world_gen.noise_frequency += 0.1;
    info!(
      "[T] Increased noise frequency to [{}]",
      settings.world_gen.noise_frequency
    );
  } else if keyboard_input.just_pressed(KeyCode::KeyG) {
    settings.world_gen.noise_frequency -= 0.1;
    info!(
      "[G] Decreased noise frequency to [{}]",
      settings.world_gen.noise_frequency
    );
  }

  // Scale factor
  if keyboard_input.just_pressed(KeyCode::KeyY) {
    settings.world_gen.elevation += 0.5;
    info!("[Y] Increased noise scale factor to [{}]", settings.world_gen.elevation);
  } else if keyboard_input.just_pressed(KeyCode::KeyH) {
    settings.world_gen.elevation -= 0.5;
    info!("[H] Decreased noise scale factor to [{}]", settings.world_gen.elevation);
  }

  // Falloff strength
  if keyboard_input.just_pressed(KeyCode::KeyU) {
    settings.world_gen.falloff_strength += 0.1;
    info!(
      "[U] Increased falloff strength to [{}]",
      settings.world_gen.falloff_strength
    );
  } else if keyboard_input.just_pressed(KeyCode::KeyJ) {
    settings.world_gen.falloff_strength -= 0.1;
    info!(
      "[J] Decreased falloff strength to [{}]",
      settings.world_gen.falloff_strength
    );
  }

  // Draw terrain sprites
  if keyboard_input.just_pressed(KeyCode::KeyZ) {
    settings.draw_terrain_sprites = !settings.draw_terrain_sprites;
    info!(
      "[Z] Toggled terrain sprite drawing to [{}]",
      settings.draw_terrain_sprites
    );
  }

  // Permit tile layer adjustments
  if keyboard_input.just_pressed(KeyCode::KeyX) {
    settings.permit_tile_layer_adjustments = !settings.permit_tile_layer_adjustments;
    info!(
      "[X] Toggled tile layer adjustments to [{}]",
      settings.permit_tile_layer_adjustments
    );
  }

  // Spawn tile debug info
  if keyboard_input.just_pressed(KeyCode::KeyC) {
    settings.spawn_tile_debug_info = !settings.spawn_tile_debug_info;
    info!("[C] Toggled tile debug info to [{}]", settings.spawn_tile_debug_info);
  }
}
