mod animation;
mod app_states;
mod camera;
mod constants;
mod controls;
mod coordinates;
mod generation;
mod shared_messages;
mod shared_resources;
mod ui;

use crate::app_states::AppStatePlugin;
use crate::camera::CameraPlugin;
use crate::constants::{WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::controls::ControlsPlugin;
use crate::generation::GenerationPipelinePlugin;
use crate::shared_messages::SharedMessagesPlugin;
use crate::shared_resources::SharedResourcesPlugin;
use crate::ui::UiPlugins;
use animation::SpriteAnimationsPlugin;
use bevy::asset::AssetMetaCheck;
use bevy::input::common_conditions::input_toggle_active;
use bevy::prelude::*;
use bevy::window::{PresentMode, WindowResolution};
use bevy_framepace::FramepacePlugin;
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

fn main() {
  App::new()
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
            present_mode: PresentMode::AutoVsync,
            resizable: false,
            ..default()
          }),
          ..default()
        })
        .build(),
    )
    .add_plugins(FramepacePlugin)
    .add_plugins((
      CameraPlugin,
      AppStatePlugin,
      GenerationPipelinePlugin,
      SpriteAnimationsPlugin,
      SharedMessagesPlugin,
      SharedResourcesPlugin,
      ControlsPlugin,
      UiPlugins,
    ))
    .add_plugins(DefaultInspectorConfigPlugin)
    .add_plugins(EguiPlugin::default())
    .add_plugins(WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::F1)))
    .run();
}
