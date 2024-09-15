use crate::events::RefreshWorldEvent;
use crate::resources::Settings;
use bevy::app::{App, Plugin, Update};
use bevy::input::ButtonInput;
use bevy::prelude::{EventWriter, KeyCode, Local, ResMut, Resource, With, World};
use bevy::window::PrimaryWindow;
use bevy_inspector_egui::bevy_egui::EguiContext;
use bevy_inspector_egui::egui::{ScrollArea, Window};

pub struct UiPlugin;

impl Plugin for UiPlugin {
  fn build(&self, app: &mut App) {
    app
      .insert_resource(UiEventsResource::default())
      .add_systems(Update, (render_settings_ui_system, handle_ui_events_system));
  }
}

#[derive(Default, Resource)]
pub struct UiEventsResource {
  pub regenerate: bool,
  pub generate_next: bool,
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
    .default_size([450.0, 400.0])
    .default_pos([10.0, 10.0])
    .show(egui_context.get_mut(), |ui| {
      ScrollArea::both().show(ui, |ui| {
        bevy_inspector_egui::bevy_inspector::ui_for_resource::<Settings>(world, ui);
        ui.separator();
        if ui.button("Regenerate").clicked() {
          let mut event_writer = world.resource_mut::<UiEventsResource>();
          event_writer.regenerate = true;
        }
        if ui.button("Generate Next").clicked() {
          let mut event_writer = world.resource_mut::<UiEventsResource>();
          event_writer.generate_next = true;
        }
        ui.separator();
        ui.label("Press F2 to toggle the inspector window");
      });
    });
}

fn handle_ui_events_system(
  mut events: EventWriter<RefreshWorldEvent>,
  mut state: ResMut<UiEventsResource>,
  mut settings: ResMut<Settings>,
) {
  if state.regenerate {
    events.send(RefreshWorldEvent {});
    state.regenerate = false;
  }

  if state.generate_next {
    settings.world_gen.noise_seed = settings.world_gen.noise_seed.saturating_add(1);
    events.send(RefreshWorldEvent {});
    state.generate_next = false;
  }
}
