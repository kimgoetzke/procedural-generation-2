use crate::coords::{Coords, Point};
use crate::events::{MouseClickEvent, RefreshWorldEvent, ToggleDebugInfo};
use crate::resources::Settings;
use bevy::app::{App, Plugin};
use bevy::prelude::*;

pub struct ControlPlugin;

impl Plugin for ControlPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(
      Update,
      (non_settings_controls_system, settings_controls_system, handle_click_system),
    );
  }
}

/**
 * A system that handles non-settings related controls. Note that the visibility of the settings UI is handled by the UI
 * plugin directly.
 */
fn non_settings_controls_system(
  keyboard_input: Res<ButtonInput<KeyCode>>,
  mut reset_world_event: EventWriter<RefreshWorldEvent>,
) {
  // Refresh world
  if keyboard_input.just_pressed(KeyCode::F5) {
    info!("[F5] Refreshing world...");
    reset_world_event.send(RefreshWorldEvent {});
  }
}

/**
 * A system that handles setting value changes.
 */
fn settings_controls_system(
  keyboard_input: Res<ButtonInput<KeyCode>>,
  mut settings: ResMut<Settings>,
  mut toggle_debug_info_event: EventWriter<ToggleDebugInfo>,
) {
  if keyboard_input.just_pressed(KeyCode::KeyZ) {
    settings.general.draw_terrain_sprites = !settings.general.draw_terrain_sprites;
    info!(
      "[Z] Toggled terrain sprite drawing to [{}]",
      settings.general.draw_terrain_sprites
    );
  }

  if keyboard_input.just_pressed(KeyCode::KeyX) {
    settings.object.object_generation = !settings.object.object_generation;
    info!("[X] Toggled object generation to [{}]", settings.object.object_generation);
  }

  if keyboard_input.just_pressed(KeyCode::KeyC) {
    settings.general.enable_tile_debugging = !settings.general.enable_tile_debugging;
    info!("[C] Toggled tile debugging to [{}]", settings.general.enable_tile_debugging);
    toggle_debug_info_event.send(ToggleDebugInfo {});
  }
}

fn handle_click_system(
  mouse_button_input: Res<ButtonInput<MouseButton>>,
  camera: Query<(&Camera, &GlobalTransform)>,
  windows: Query<&Window>,
  mut commands: Commands,
) {
  if mouse_button_input.just_pressed(MouseButton::Left) {
    let (camera, camera_transform) = camera.single();
    if let Some(vec2) = windows
      .single()
      .cursor_position()
      .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
      .map(|ray| ray.origin.truncate())
    {
      let world_grid = Point::new_world_grid_from_world_vec2(vec2);
      let chunk_grid = Point::new_chunk_grid_from_world_vec2(vec2);
      let coords = Coords::new(world_grid, chunk_grid);
      info!(
        "[Left Mouse Button] Clicked on w{:?} wg{:?}...",
        coords.world, coords.world_grid
      );
      commands.trigger(MouseClickEvent { coords });
    }
  }
}
