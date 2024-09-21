use crate::events::RefreshWorldEvent;
use crate::resources::{GeneralGenerationSettings, Settings, WorldGenerationSettings};
use bevy::app::{App, Plugin, Update};
use bevy::input::ButtonInput;
use bevy::prelude::{EventWriter, KeyCode, Local, Res, ResMut, Resource, With, World};
use bevy::window::PrimaryWindow;
use bevy_inspector_egui::bevy_egui::EguiContext;
use bevy_inspector_egui::egui::{Align, Align2, FontId, Layout, RichText, ScrollArea, Window};

const HEADING: FontId = FontId::proportional(16.0);

pub struct UiPlugin;

impl Plugin for UiPlugin {
  fn build(&self, app: &mut App) {
    app
      .insert_resource(UiEventsResource::default())
      .add_systems(Update, (render_settings_ui_system, handle_ui_events_system));
  }
}

#[derive(Default, Resource)]
struct UiEventsResource {
  has_changed: bool,
  regenerate: bool,
  generate_next: bool,
}

impl UiEventsResource {
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
    .default_size([325.0, 400.0])
    .pivot(Align2::LEFT_BOTTOM)
    .anchor(Align2::LEFT_BOTTOM, [10.0, -10.0])
    .show(egui_context.get_mut(), |ui| {
      ScrollArea::both().show(ui, |ui| {
        ui.label(RichText::new("General Generation").font(HEADING));
        bevy_inspector_egui::bevy_inspector::ui_for_resource::<GeneralGenerationSettings>(world, ui);
        ui.add_space(20.0);
        ui.label(RichText::new("World Generation").font(HEADING));
        ui.vertical(|ui| {
          bevy_inspector_egui::bevy_inspector::ui_for_resource::<WorldGenerationSettings>(world, ui);
        });
        ui.separator();
        ui.horizontal(|ui| {
          if ui.button("Regenerate").clicked() {
            let mut event_writer = world.resource_mut::<UiEventsResource>();
            event_writer.trigger_regeneration();
          }
          ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui.button("Generate Next").clicked() {
              let mut event_writer = world.resource_mut::<UiEventsResource>();
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
  mut events: EventWriter<RefreshWorldEvent>,
  mut state: ResMut<UiEventsResource>,
  mut settings: ResMut<Settings>,
  general: Res<GeneralGenerationSettings>,
  mut world_gen: ResMut<WorldGenerationSettings>,
) {
  if state.has_changed {
    state.has_changed = false;
    settings.general = general.clone();
    settings.world = world_gen.clone();

    if state.regenerate {
      events.send(RefreshWorldEvent {});
      state.regenerate = false;
    }

    if state.generate_next {
      settings.world.noise_seed = settings.world.noise_seed.saturating_add(1);
      world_gen.noise_seed = settings.world.noise_seed;
      events.send(RefreshWorldEvent {});
      state.generate_next = false;
    }
  }
}
