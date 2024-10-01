use crate::constants::ORIGIN_WORLD_GRID_SPAWN_POINT;
use crate::coords::Point;
use crate::events::{DespawnChunksEvent, RegenerateWorldEvent, UpdateWorldEvent};
use crate::resources::{
  CurrentChunk, GeneralGenerationSettings, ObjectGenerationSettings, Settings, WorldGenerationSettings,
};
use bevy::app::{App, Plugin, Update};
use bevy::input::ButtonInput;
use bevy::log::*;
use bevy::math::Vec3;
use bevy::prelude::{Camera, EventWriter, GlobalTransform, KeyCode, Local, Query, Res, ResMut, Resource, With, World};
use bevy::window::PrimaryWindow;
use bevy_inspector_egui::bevy_egui::EguiContext;
use bevy_inspector_egui::egui::{Align, Align2, FontId, Layout, RichText, ScrollArea, Window};

const HEADING: FontId = FontId::proportional(16.0);

pub struct UiPlugin;

impl Plugin for UiPlugin {
  fn build(&self, app: &mut App) {
    app
      .insert_resource(UiState::default())
      .add_systems(Update, (render_settings_ui_system, handle_ui_events_system));
  }
}

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

  let mut egui_context = world
    .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
    .single(world)
    .clone();

  Window::new("Settings")
    .default_size([325.0, 500.0])
    .pivot(Align2::LEFT_BOTTOM)
    .anchor(Align2::LEFT_BOTTOM, [10.0, -10.0])
    .show(egui_context.get_mut(), |ui| {
      ScrollArea::both().show(ui, |ui| {
        ui.push_id("general_generation", |ui| {
          ui.label(RichText::new("General Generation").font(HEADING));
          bevy_inspector_egui::bevy_inspector::ui_for_resource::<GeneralGenerationSettings>(world, ui);
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
  mut regenerate_event: EventWriter<RegenerateWorldEvent>,
  mut update_event: EventWriter<UpdateWorldEvent>,
  mut despawn_chunks_event: EventWriter<DespawnChunksEvent>,
  mut state: ResMut<UiState>,
  mut settings: ResMut<Settings>,
  general: Res<GeneralGenerationSettings>,
  object: Res<ObjectGenerationSettings>,
  mut world_gen: ResMut<WorldGenerationSettings>,
  current_chunk: Res<CurrentChunk>,
  camera: Query<(&Camera, &GlobalTransform)>,
) {
  if state.has_changed {
    state.has_changed = false;
    settings.general = general.clone();
    settings.world = world_gen.clone();
    settings.object = object.clone();
    let camera_translation = camera.single().1.translation();

    if state.regenerate {
      send_regenerate_or_update_event(
        &mut regenerate_event,
        &mut update_event,
        &mut despawn_chunks_event,
        &current_chunk,
        camera_translation,
      );
      state.regenerate = false;
    }

    if state.generate_next {
      settings.world.noise_seed = settings.world.noise_seed.saturating_add(1);
      world_gen.noise_seed = settings.world.noise_seed;
      send_regenerate_or_update_event(
        &mut regenerate_event,
        &mut update_event,
        &mut despawn_chunks_event,
        &current_chunk,
        camera_translation,
      );
      debug!("Generating next world with seed {}", settings.world.noise_seed);
      state.generate_next = false;
    }
  }
}

fn send_regenerate_or_update_event(
  regenerate_event: &mut EventWriter<RegenerateWorldEvent>,
  update_event: &mut EventWriter<UpdateWorldEvent>,
  despawn_chunks_event: &mut EventWriter<DespawnChunksEvent>,
  current_chunk: &Res<CurrentChunk>,
  camera_translation: Vec3,
) {
  if current_chunk.get_world_grid() == ORIGIN_WORLD_GRID_SPAWN_POINT {
    regenerate_event.send(RegenerateWorldEvent {});
  } else {
    let current_world = Point::new_world_from_world_vec2(camera_translation.truncate());
    despawn_chunks_event.send(DespawnChunksEvent { despawn_all: true });
    update_event.send(UpdateWorldEvent {
      is_forced_update: true,
      world_grid: Point::new_world_grid_from_world(current_world),
      world: current_world,
    });
  }
}
