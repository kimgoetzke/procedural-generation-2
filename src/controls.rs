use crate::constants::{CHUNK_SIZE, ORIGIN_WORLD_GRID_SPAWN_POINT, TILE_SIZE};
use crate::coords::Point;
use crate::events::{MouseClickEvent, PruneWorldEvent, RegenerateWorldEvent, ToggleDebugInfo, UpdateWorldEvent};
use crate::resources::{CurrentChunk, GeneralGenerationSettings, ObjectGenerationSettings, Settings};
use bevy::app::{App, Plugin};
use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiContexts;

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

fn event_control_system(
  keyboard_input: Res<ButtonInput<KeyCode>>,
  mut regenerate_event: EventWriter<RegenerateWorldEvent>,
  mut prune_event: EventWriter<PruneWorldEvent>,
  current_chunk: Res<CurrentChunk>,
) {
  if keyboard_input.just_pressed(KeyCode::F5) | keyboard_input.just_pressed(KeyCode::KeyR) {
    info!("[F5]/[R] Triggered regeneration of the world");
    if current_chunk.get_world_grid() == ORIGIN_WORLD_GRID_SPAWN_POINT {
      regenerate_event.send(RegenerateWorldEvent {});
    } else {
      prune_event.send(PruneWorldEvent {
        despawn_all_chunks: true,
        update_world_after: true,
      });
    }
  }
}

fn settings_controls_system(
  keyboard_input: Res<ButtonInput<KeyCode>>,
  mut settings: ResMut<Settings>,
  mut general_settings: ResMut<GeneralGenerationSettings>,
  mut object_settings: ResMut<ObjectGenerationSettings>,
  mut toggle_debug_info_event: EventWriter<ToggleDebugInfo>,
) {
  if keyboard_input.just_pressed(KeyCode::KeyZ) {
    settings.general.draw_gizmos = !settings.general.draw_gizmos;
    general_settings.draw_gizmos = settings.general.draw_gizmos;
    info!("[Z] Set drawing gizmos to [{}]", settings.general.draw_gizmos);
  }

  if keyboard_input.just_pressed(KeyCode::KeyX) {
    settings.general.generate_neighbour_chunks = !settings.general.generate_neighbour_chunks;
    general_settings.generate_neighbour_chunks = settings.general.generate_neighbour_chunks;
    info!(
      "[X] Set generating neighbour chunks to [{}]",
      settings.general.generate_neighbour_chunks
    );
  }

  if keyboard_input.just_pressed(KeyCode::KeyC) {
    settings.general.enable_tile_debugging = !settings.general.enable_tile_debugging;
    general_settings.enable_tile_debugging = settings.general.enable_tile_debugging;
    info!("[C] Set tile debugging to [{}]", settings.general.enable_tile_debugging);
    toggle_debug_info_event.send(ToggleDebugInfo {});
  }

  if keyboard_input.just_pressed(KeyCode::KeyV) {
    settings.general.draw_terrain_sprites = !settings.general.draw_terrain_sprites;
    general_settings.draw_terrain_sprites = settings.general.draw_terrain_sprites;
    info!(
      "[V] Set drawing terrain sprites to [{}]",
      settings.general.draw_terrain_sprites
    );
  }

  if keyboard_input.just_pressed(KeyCode::KeyB) {
    settings.general.animate_terrain_sprites = !settings.general.animate_terrain_sprites;
    general_settings.animate_terrain_sprites = settings.general.animate_terrain_sprites;
    info!(
      "[V] Set animating terrain sprites to [{}]",
      settings.general.animate_terrain_sprites
    );
  }

  if keyboard_input.just_pressed(KeyCode::KeyF) {
    settings.object.generate_objects = !settings.object.generate_objects;
    object_settings.generate_objects = settings.object.generate_objects;
    info!("[F] Set object generation to [{}]", settings.object.generate_objects);
  }
}

fn left_mouse_click_system(
  mouse_button_input: Res<ButtonInput<MouseButton>>,
  camera: Query<(&Camera, &GlobalTransform)>,
  windows: Query<&Window>,
  mut commands: Commands,
  mut egui_contexts: EguiContexts,
) {
  if mouse_button_input.just_pressed(MouseButton::Left) && !egui_contexts.ctx_mut().wants_pointer_input() {
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
  mut event: EventWriter<UpdateWorldEvent>,
) {
  let translation = camera.single().1.translation();
  let current_world = Point::new_world_from_world_vec2(translation.truncate());
  let chunk_center_world = current_chunk.get_center_world();
  let distance_x = (current_world.x - chunk_center_world.x).abs();
  let distance_y = (current_world.y - chunk_center_world.y).abs();
  let trigger_distance = ((CHUNK_SIZE * TILE_SIZE as i32) / 2) + 1;
  trace!(
    "Camera moved to w{:?} with distance x={:?}, y={:?} (trigger distance {})",
    current_world,
    distance_x,
    distance_y,
    trigger_distance
  );

  if (distance_x >= trigger_distance) || (distance_y >= trigger_distance) {
    event.send(UpdateWorldEvent {
      is_forced_update: false,
      world_grid: Point::new_world_grid_from_world(current_world),
      world: current_world,
    });
  };
}
