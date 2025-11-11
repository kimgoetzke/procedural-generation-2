use crate::constants::*;
use crate::messages::{RefreshMetadataMessage, ResetCameraMessage, ToggleDebugInfoMessage, ToggleDiagnosticsMessage};
use crate::resources::{
  CurrentChunk, GeneralGenerationSettings, GenerationMetadataSettings, ObjectGenerationSettings, Settings,
  WorldGenerationSettings,
};
use crate::states::{AppState, GenerationState};
use bevy::app::{App, Plugin, Update};
use bevy::input::ButtonInput;
use bevy::prelude::{KeyCode, Local, MessageWriter, Res, ResMut, Resource, With, World};
use bevy_inspector_egui::bevy_egui::{EguiContext, EguiPrimaryContextPass, PrimaryEguiContext};
use bevy_inspector_egui::egui::{Align, Align2, Color32, FontId, Layout, RichText, ScrollArea, Window};

pub struct SettingsUiPlugin;

impl Plugin for SettingsUiPlugin {
  fn build(&self, app: &mut App) {
    app
      .insert_resource(UiState::default())
      .add_systems(Update, handle_ui_messages_system)
      .add_systems(EguiPrimaryContextPass, render_settings_ui_system);
  }
}

const HEADING: FontId = FontId::proportional(16.0);
const COMMENT: FontId = FontId::proportional(12.0);

#[derive(Default, Resource)]
struct UiState {
  pending_action: Option<UiAction>,
}

impl UiState {
  fn trigger_action(&mut self, action: UiAction) {
    self.pending_action = Some(action);
  }

  fn take_action(&mut self) -> Option<UiAction> {
    self.pending_action.take()
  }
}

#[derive(Debug, Clone)]
enum UiAction {
  Regenerate,
  GenerateNext,
  ResetSettings,
  ResetCamera,
}

fn render_settings_ui_system(world: &mut World, mut disabled: Local<bool>) {
  let is_toggled = world.resource::<ButtonInput<KeyCode>>().just_pressed(KeyCode::F2);
  if is_toggled {
    *disabled = !*disabled;
  }
  if *disabled {
    return;
  }

  let mut egui_context = if let Ok(context) = world
    .query_filtered::<&mut EguiContext, With<PrimaryEguiContext>>()
    .single(world)
  {
    context.clone()
  } else {
    panic!("No egui context found");
  };

  // Increase the default tooltip width in order to allow documentation comments to be displayed without double
  // line-breaking - if this ever breaks the UI, remove it and use single line comments for settings instead
  egui_context.get_mut().style_mut(|style| {
    style.spacing.tooltip_width = 700.0;
  });

  Window::new("Settings")
    .default_size([370.0, 600.0])
    .pivot(Align2::LEFT_BOTTOM)
    .anchor(Align2::LEFT_BOTTOM, [10.0, -10.0])
    .show(egui_context.get_mut(), |ui| {
      ScrollArea::both().show(ui, |ui| {
        render_states_section(world, ui);
        ui.add_space(20.0);
        ui.push_id("general_generation", |ui| {
          ui.label(RichText::new("General Generation").font(HEADING));
          bevy_inspector_egui::bevy_inspector::ui_for_resource::<GeneralGenerationSettings>(world, ui);
        });
        ui.add_space(20.0);
        ui.push_id("generation_metadata", |ui| {
          ui.label(RichText::new("Generation Metadata").font(HEADING));
          ui.label(RichText::new("Metadata settings can easily cause rendering issues if misconfigured, such as misaligned chunks. No safeguards have been implemented yet. Use with care.")
            .font(COMMENT)
            .italics());
          bevy_inspector_egui::bevy_inspector::ui_for_resource::<GenerationMetadataSettings>(world, ui);
        });
        ui.add_space(20.0);
        ui.push_id("world_generation", |ui| {
          ui.label(RichText::new("World Generation").font(HEADING));
          bevy_inspector_egui::bevy_inspector::ui_for_resource::<WorldGenerationSettings>(world, ui);
        });
        ui.add_space(20.0);
        ui.push_id("object_generation", |ui| {
          ui.label(RichText::new("Object Generation").font(HEADING));
          bevy_inspector_egui::bevy_inspector::ui_for_resource::<ObjectGenerationSettings>(world, ui);
        });
        ui.add_space(20.0);
        ui.label(RichText::new("You must hit [Regenerate] to apply any changes to the above.")
          .font(COMMENT)
          .italics());
        ui.separator();
        render_buttons(world, ui);
        ui.separator();
        ui.label("Press F2 to toggle the inspector window");
      });
    });
}

fn render_states_section(world: &mut World, ui: &mut bevy_inspector_egui::egui::Ui) {
  ui.push_id("states", |ui| {
    ui.label(RichText::new("States").font(HEADING));
    ui.columns(2, |columns| {
      columns[0].label("app_state");
      columns[1].push_id("app_state", |ui| {
        bevy_inspector_egui::bevy_inspector::ui_for_state::<AppState>(world, ui)
      });
    });
    ui.columns(2, |columns| {
      columns[0].label("generation_state");
      columns[1].push_id("generation_state", |ui| {
        bevy_inspector_egui::bevy_inspector::ui_for_state::<GenerationState>(world, ui)
      });
    });
  });
}

fn render_buttons(world: &mut World, ui: &mut bevy_inspector_egui::egui::Ui) {
  let very_dark = crate::generation::lib::shared::to_colour_32(VERY_DARK);
  let red = crate::generation::lib::shared::to_colour_32(RED);
  ui.horizontal(|ui| {
    ui.style_mut().visuals.widgets.hovered.weak_bg_fill = red;
    if ui.button("Reset Settings")
      .on_hover_text("Resets all settings that can be changed at run-time and regenerates the world.")
      .clicked()
    {
      world.resource_mut::<UiState>().trigger_action(UiAction::ResetSettings);
    }
    if ui.button("Reset Camera")
      .on_hover_text("Resets the camera position/zoom and regenerates the world without changing any settings.")
      .clicked()
    {
      world.resource_mut::<UiState>().trigger_action(UiAction::ResetCamera);
    }
    ui.style_mut().visuals.widgets.hovered.weak_bg_fill = Color32::PLACEHOLDER;
    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
      ui.style_mut().visuals.widgets.hovered.weak_bg_fill = very_dark;
      if ui.button("Generate Next")
        .on_hover_text("Increments the world generation seed and generates the new world without changing any other settings or the camera position/zoom level.")
        .clicked()
      {
        world.resource_mut::<UiState>().trigger_action(UiAction::GenerateNext);
      }
      if ui.button("Regenerate")
        .on_hover_text("Regenerates the entire world without changing any settings or the camera position/zoom level.")
        .clicked()
      {
        world.resource_mut::<UiState>().trigger_action(UiAction::Regenerate);
      }
      ui.style_mut().visuals.widgets.hovered.weak_bg_fill = Color32::PLACEHOLDER;
    });
  });
}

fn handle_ui_messages_system(
  mut ui_state: ResMut<UiState>,
  mut settings: ResMut<Settings>,
  mut general: ResMut<GeneralGenerationSettings>,
  mut metadata_settings: ResMut<GenerationMetadataSettings>,
  mut object: ResMut<ObjectGenerationSettings>,
  mut world_gen: ResMut<WorldGenerationSettings>,
  current_chunk: Res<CurrentChunk>,
  mut refresh_metadata_message: MessageWriter<RefreshMetadataMessage>,
  mut reset_camera_message: MessageWriter<ResetCameraMessage>,
  mut toggle_debug_info_message: MessageWriter<ToggleDebugInfoMessage>,
  mut toggle_diagnostics_message: MessageWriter<ToggleDiagnosticsMessage>,
) {
  if let Some(action) = ui_state.take_action() {
    match action {
      UiAction::ResetSettings => {
        reset_all_settings(
          &mut settings,
          &mut general,
          &mut metadata_settings,
          &mut world_gen,
          &mut object,
        );
        send_regenerate_or_prune_message(&current_chunk, &mut refresh_metadata_message);
      }
      UiAction::ResetCamera => {
        reset_camera_message.write(ResetCameraMessage { reset_position: true });
        send_regenerate_or_prune_message(&current_chunk, &mut refresh_metadata_message);
      }
      UiAction::Regenerate => {
        update_settings(&mut settings, &general, &metadata_settings, &world_gen, &object);
        send_regenerate_or_prune_message(&current_chunk, &mut refresh_metadata_message);
      }
      UiAction::GenerateNext => {
        update_settings(&mut settings, &general, &metadata_settings, &world_gen, &object);
        settings.world.noise_seed = settings.world.noise_seed.saturating_add(1);
        world_gen.noise_seed = settings.world.noise_seed;
        send_regenerate_or_prune_message(&current_chunk, &mut refresh_metadata_message);
      }
    }
    toggle_debug_info_message.write(ToggleDebugInfoMessage {});
    toggle_diagnostics_message.write(ToggleDiagnosticsMessage {});
  }
}

fn reset_all_settings(
  settings: &mut ResMut<Settings>,
  general: &mut ResMut<GeneralGenerationSettings>,
  metadata_settings: &mut ResMut<GenerationMetadataSettings>,
  world_gen: &mut ResMut<WorldGenerationSettings>,
  object: &mut ResMut<ObjectGenerationSettings>,
) {
  let (default_general, default_metadata, default_world, default_object) = (
    GeneralGenerationSettings::default(),
    GenerationMetadataSettings::default(),
    WorldGenerationSettings::default(),
    ObjectGenerationSettings::default(),
  );
  settings.general = default_general;
  settings.metadata = default_metadata;
  settings.world = default_world;
  settings.object = default_object;
  **general = default_general;
  **metadata_settings = default_metadata;
  **world_gen = default_world;
  **object = default_object;
}

fn update_settings(
  settings: &mut ResMut<Settings>,
  general: &GeneralGenerationSettings,
  metadata_settings: &GenerationMetadataSettings,
  world_gen: &WorldGenerationSettings,
  object: &ObjectGenerationSettings,
) {
  settings.general = *general;
  settings.metadata = *metadata_settings;
  settings.world = *world_gen;
  settings.object = *object;
}

fn send_regenerate_or_prune_message(
  current_chunk: &Res<CurrentChunk>,
  refresh_metadata_message: &mut MessageWriter<RefreshMetadataMessage>,
) {
  let is_at_origin_spawn_point = current_chunk.get_tile_grid() == ORIGIN_TILE_GRID_SPAWN_POINT;
  refresh_metadata_message.write(RefreshMetadataMessage {
    regenerate_world_after: is_at_origin_spawn_point,
    prune_then_update_world_after: !is_at_origin_spawn_point,
  });
}
