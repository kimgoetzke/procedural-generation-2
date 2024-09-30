use crate::constants::{CHUNK_SIZE, TILE_SIZE};
use crate::coords::Point;
use crate::events::{ChunkGenerationEvent, MouseClickEvent, RefreshWorldEvent, ToggleDebugInfo};
use crate::resources::{CurrentChunk, Settings};
use bevy::app::{App, Plugin};
use bevy::prelude::*;

pub struct ControlPlugin;

impl Plugin for ControlPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(
      Update,
      (
        event_control_system,
        settings_controls_system,
        left_mouse_click_system,
        camera_movement_system,
      ),
    );
  }
}

fn event_control_system(keyboard_input: Res<ButtonInput<KeyCode>>, mut reset_world_event: EventWriter<RefreshWorldEvent>) {
  // Refresh world
  if keyboard_input.just_pressed(KeyCode::F5) {
    info!("[F5] Refreshing world...");
    reset_world_event.send(RefreshWorldEvent {});
  }
}

/// A system that handles setting value changes via keyboard inputs.
fn settings_controls_system(
  keyboard_input: Res<ButtonInput<KeyCode>>,
  mut settings: ResMut<Settings>,
  mut toggle_debug_info_event: EventWriter<ToggleDebugInfo>,
) {
  if keyboard_input.just_pressed(KeyCode::KeyZ) {
    settings.general.draw_terrain_sprites = !settings.general.draw_terrain_sprites;
    info!(
      "[Z] Set drawing terrain sprites to [{}]",
      settings.general.draw_terrain_sprites
    );
  }

  if keyboard_input.just_pressed(KeyCode::KeyX) {
    settings.object.object_generation = !settings.object.object_generation;
    info!("[X] Set object generation to [{}]", settings.object.object_generation);
  }

  if keyboard_input.just_pressed(KeyCode::KeyC) {
    settings.general.enable_tile_debugging = !settings.general.enable_tile_debugging;
    info!("[C] Set tile debugging to [{}]", settings.general.enable_tile_debugging);
    toggle_debug_info_event.send(ToggleDebugInfo {});
  }

  if keyboard_input.just_pressed(KeyCode::KeyV) {
    settings.general.draw_gizmos = !settings.general.draw_gizmos;
    info!("[V] Set drawing gizmos to [{}]", settings.general.draw_gizmos);
    toggle_debug_info_event.send(ToggleDebugInfo {});
  }
}

fn left_mouse_click_system(
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
      let world = Point::new_world_from_world_grid(world_grid);
      debug!(
        "[Left Mouse Button] Clicked on {} => w{:?} wg{:?}",
        vec2.round(),
        world,
        world_grid
      );
      commands.trigger(MouseClickEvent { world, world_grid });
    }
  }
}

fn camera_movement_system(
  camera: Query<(&Camera, &GlobalTransform)>,
  current_chunk: Res<CurrentChunk>,
  mut event: EventWriter<ChunkGenerationEvent>,
) {
  let point = camera.single().1.translation();
  let current_world = Point::new_world_from_world_vec2(point.truncate());
  let chunk_center_world = current_chunk.get_center_world();
  let distance_x = (current_world.x - chunk_center_world.x).abs();
  let distance_y = (current_world.y - chunk_center_world.y).abs();
  trace!(
    "Camera moved to w{:?} with distance ({:?}, {:?})",
    current_world,
    distance_x,
    distance_y
  );

  if (distance_x >= (CHUNK_SIZE * TILE_SIZE as i32) / 2) || (distance_y >= (CHUNK_SIZE * TILE_SIZE as i32) / 2) {
    event.send(ChunkGenerationEvent {
      world: current_world,
      world_grid: Point::new_world_grid_from_world(current_world),
    });
  };
}
