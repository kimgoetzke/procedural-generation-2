mod camera;
mod controls;
mod events;
mod resources;
mod settings;
mod world;

use crate::camera::CameraPlugin;
use crate::controls::ControlPlugin;
use crate::events::SharedEventsPlugin;
use crate::resources::SharedResourcesPlugin;
use crate::settings::{WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::world::WorldPlugin;
use bevy::asset::AssetMetaCheck;
use bevy::audio::{AudioPlugin, SpatialScale};
#[cfg(feature = "dev")]
use bevy::input::common_conditions::input_toggle_active;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::window::WindowResolution;
#[cfg(feature = "dev")]
use bevy_inspector_egui::quick::WorldInspectorPlugin;
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
    .add_plugins(CameraPlugin)
    .add_plugins((WorldPlugin, SharedEventsPlugin, SharedResourcesPlugin, ControlPlugin));

  #[cfg(feature = "dev")]
  app.add_plugins(WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::F1)));

  app.run();
}
