use crate::constants::ORIGIN_TILE_GRID_SPAWN_POINT;
use crate::events::RefreshMetadata;
use crate::resources::{
  CurrentChunk, GeneralGenerationSettings, GenerationMetadataSettings, ObjectGenerationSettings, Settings,
  WorldGenerationSettings,
};
use crate::states::{AppState, GenerationState};
use bevy::app::{App, Plugin, Update};
use bevy::input::ButtonInput;
use bevy::log::*;
use bevy::prelude::{EventWriter, KeyCode, Local, Res, ResMut, Resource, With, World};
use bevy::window::PrimaryWindow;
use bevy_inspector_egui::bevy_egui::EguiContext;
use bevy_inspector_egui::egui::{Align, Align2, FontId, Layout, RichText, ScrollArea, Window};

pub struct SettingsUiPlugin;

impl Plugin for SettingsUiPlugin {
  fn build(&self, app: &mut App) {
    app
      .insert_resource(UiState::default())
      .add_systems(Update, (render_settings_ui_system, handle_ui_events_system));
  }
}

const HEADING: FontId = FontId::proportional(16.0);

#[derive(Default, Resource)]
struct UiState {
  has_changed: bool,
  regenerate: bool,
  generate_next: bool,
}

impl UiState {
  pub fn trigger_regeneration(&mut self) {
    self.regenerate = true;
    self.has_changed = true;
  }

  pub fn trigger_next_generation(&mut self) {
    self.generate_next = true;
    self.has_changed = true;
  }
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
    .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
    .get_single(world)
  {
    context.clone()
  } else {
    warn_once!("No egui context found");
    return;
  };

  Window::new("Settings")
    .default_size([340.0, 600.0])
    .pivot(Align2::LEFT_BOTTOM)
    .anchor(Align2::LEFT_BOTTOM, [10.0, -10.0])
    .show(egui_context.get_mut(), |ui| {
      ScrollArea::both().show(ui, |ui| {
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
        ui.add_space(20.0);
        ui.push_id("general_generation", |ui| {
          ui.label(RichText::new("General Generation").font(HEADING));
          bevy_inspector_egui::bevy_inspector::ui_for_resource::<GeneralGenerationSettings>(world, ui);
        });
        ui.add_space(20.0);
        ui.push_id("generation_metadata", |ui| {
          ui.label(RichText::new("Generation Metadata").font(HEADING));
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
        ui.separator();
        ui.horizontal(|ui| {
          if ui.button("Regenerate").clicked() {
            let mut event_writer = world.resource_mut::<UiState>();
            event_writer.trigger_regeneration();
          }
          ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui.button("Generate Next").clicked() {
              let mut event_writer = world.resource_mut::<UiState>();
              event_writer.trigger_next_generation();
            }
          });
        });
        ui.separator();
        ui.label("Press F2 to toggle the inspector window");
      });
    });
}

fn handle_ui_events_system(
  mut refresh_metadata_event: EventWriter<RefreshMetadata>,
  mut state: ResMut<UiState>,
  mut settings: ResMut<Settings>,
  general: Res<GeneralGenerationSettings>,
  metadata_settings: Res<GenerationMetadataSettings>,
  object: Res<ObjectGenerationSettings>,
  mut world_gen: ResMut<WorldGenerationSettings>,
  current_chunk: Res<CurrentChunk>,
) {
  if state.has_changed {
    state.has_changed = false;
    settings.general = general.clone();
    settings.metadata = metadata_settings.clone();
    settings.world = world_gen.clone();
    settings.object = object.clone();

    if state.regenerate {
      send_regenerate_or_prune_event(&current_chunk, &mut refresh_metadata_event);
      state.regenerate = false;
    }

    if state.generate_next {
      settings.world.noise_seed = settings.world.noise_seed.saturating_add(1);
      world_gen.noise_seed = settings.world.noise_seed;
      send_regenerate_or_prune_event(&current_chunk, &mut refresh_metadata_event);
      state.generate_next = false;
    }
  }
}

fn send_regenerate_or_prune_event(
  current_chunk: &Res<CurrentChunk>,
  refresh_metadata_event: &mut EventWriter<RefreshMetadata>,
) {
  let is_at_origin_spawn_point = current_chunk.get_tile_grid() == ORIGIN_TILE_GRID_SPAWN_POINT;
  refresh_metadata_event.send(RefreshMetadata {
    regenerate_world_after: is_at_origin_spawn_point,
    prune_then_update_world_after: !is_at_origin_spawn_point,
  });
}
