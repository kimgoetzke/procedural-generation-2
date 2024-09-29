mod camera;
mod constants;
mod controls;
mod coords;
mod events;
mod resources;
mod ui;
mod world;

use crate::camera::CameraPlugin;
use crate::constants::{WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::controls::ControlPlugin;
use crate::events::SharedEventsPlugin;
use crate::resources::SharedResourcesPlugin;
use crate::ui::UiPlugin;
use crate::world::GenerationPlugin;
use bevy::asset::AssetMetaCheck;
use bevy::audio::{AudioPlugin, SpatialScale};
use bevy::input::common_conditions::input_toggle_active;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use bevy_pancam::PanCamPlugin;

fn main() {
  let mut app = App::new();
  app
    .add_plugins(
      DefaultPlugins
        .set(AssetPlugin {
          // This is a workaround for https://github.com/bevyengine/bevy/issues/10157
          meta_check: AssetMetaCheck::Never,
          ..default()
        })
        .set(ImagePlugin::default_nearest())
        .set(WindowPlugin {
          primary_window: Some(Window {
            title: "Procedural Generation 2".into(),
            resolution: WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT),
            resizable: false,
            ..default()
          }),
          ..default()
        })
        .set(AudioPlugin {
          default_spatial_scale: SpatialScale::new_2d(0.005),
          ..default()
        })
        .set(LogPlugin::default())
        .build(),
    )
    .add_plugins(PanCamPlugin::default())
    .add_plugins((
      CameraPlugin,
      GenerationPlugin,
      SharedEventsPlugin,
      SharedResourcesPlugin,
      ControlPlugin,
      UiPlugin,
    ));

  app
    .add_plugins(DefaultInspectorConfigPlugin)
    .add_plugins(WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::F1)));

  app.run();
}
