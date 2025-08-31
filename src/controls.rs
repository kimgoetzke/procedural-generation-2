use crate::constants::ORIGIN_TILE_GRID_SPAWN_POINT;
use crate::coords::Point;
use crate::events::{MouseClickEvent, RefreshMetadata, ToggleDebugInfo, ToggleDiagnostics};
use crate::resources::{CurrentChunk, GeneralGenerationSettings, ObjectGenerationSettings, Settings};
use bevy::app::{App, Plugin};
use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiContexts;

pub struct ControlPlugin;

impl Plugin for ControlPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(
      Update,
      (event_control_system, settings_controls_system, left_mouse_click_system),
    );
  }
}

fn event_control_system(
  keyboard_input: Res<ButtonInput<KeyCode>>,
  mut refresh_metadata_event: EventWriter<RefreshMetadata>,
  current_chunk: Res<CurrentChunk>,
) {
  if keyboard_input.just_pressed(KeyCode::F5) | keyboard_input.just_pressed(KeyCode::KeyR) {
    info!("[F5]/[R] Triggered regeneration of the world");
    let is_at_origin_spawn_point = current_chunk.get_tile_grid() == ORIGIN_TILE_GRID_SPAWN_POINT;
    refresh_metadata_event.write(RefreshMetadata {
      regenerate_world_after: is_at_origin_spawn_point,
      prune_then_update_world_after: !is_at_origin_spawn_point,
    });
  }
}

fn settings_controls_system(
  keyboard_input: Res<ButtonInput<KeyCode>>,
  mut settings: ResMut<Settings>,
  mut general_settings: ResMut<GeneralGenerationSettings>,
  mut object_settings: ResMut<ObjectGenerationSettings>,
  mut toggle_debug_info_event: EventWriter<ToggleDebugInfo>,
  mut toggle_diagnostics_event: EventWriter<ToggleDiagnostics>,
) {
  if keyboard_input.just_pressed(KeyCode::KeyZ) {
    settings.general.display_diagnostics = !settings.general.display_diagnostics;
    general_settings.display_diagnostics = settings.general.display_diagnostics;
    info!("[Z] Set display diagnostics to [{}]", settings.general.display_diagnostics);
    toggle_diagnostics_event.write(ToggleDiagnostics {});
  }

  if keyboard_input.just_pressed(KeyCode::KeyX) {
    settings.general.draw_gizmos = !settings.general.draw_gizmos;
    general_settings.draw_gizmos = settings.general.draw_gizmos;
    info!("[X] Set drawing gizmos to [{}]", settings.general.draw_gizmos);
  }

  if keyboard_input.just_pressed(KeyCode::KeyC) {
    settings.general.enable_tile_debugging = !settings.general.enable_tile_debugging;
    general_settings.enable_tile_debugging = settings.general.enable_tile_debugging;
    info!("[C] Set tile debugging to [{}]", settings.general.enable_tile_debugging);
    toggle_debug_info_event.write(ToggleDebugInfo {});
  }

  if keyboard_input.just_pressed(KeyCode::KeyV) {
    settings.general.generate_neighbour_chunks = !settings.general.generate_neighbour_chunks;
    general_settings.generate_neighbour_chunks = settings.general.generate_neighbour_chunks;
    info!(
      "[V] Set generating neighbour chunks to [{}]",
      settings.general.generate_neighbour_chunks
    );
  }

  if keyboard_input.just_pressed(KeyCode::KeyB) {
    settings.general.draw_terrain_sprites = !settings.general.draw_terrain_sprites;
    general_settings.draw_terrain_sprites = settings.general.draw_terrain_sprites;
    info!(
      "[B] Set drawing terrain sprites to [{}]",
      settings.general.draw_terrain_sprites
    );
  }

  if keyboard_input.just_pressed(KeyCode::KeyN) {
    settings.general.animate_terrain_sprites = !settings.general.animate_terrain_sprites;
    general_settings.animate_terrain_sprites = settings.general.animate_terrain_sprites;
    info!(
      "[N] Set animating terrain sprites to [{}]",
      settings.general.animate_terrain_sprites
    );
  }

  if keyboard_input.just_pressed(KeyCode::KeyM) {
    settings.general.enable_world_pruning = !settings.general.enable_world_pruning;
    general_settings.enable_world_pruning = settings.general.enable_world_pruning;
    info!("[M] Set world pruning to [{}]", settings.general.enable_world_pruning);
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
    let (camera, camera_transform) = camera.single().expect("Failed to find camera");
    if let Some(vec2) = windows
      .single()
      .expect("Failed to find window")
      .cursor_position()
      .and_then(|cursor| Some(camera.viewport_to_world(camera_transform, cursor)))
      .map(|ray| ray.expect("Failed to find ray").origin.truncate())
    {
      let tg = Point::new_tile_grid_from_world_vec2(vec2);
      let cg = Point::new_chunk_grid_from_world_vec2(vec2);
      let tile_w = Point::new_world_from_tile_grid(tg);
      debug!("[Left Mouse Button] Clicked on {} => {} {} {}", vec2.round(), tile_w, cg, tg);
      commands.trigger(MouseClickEvent { tile_w, cg, tg });
    }
  }
}
