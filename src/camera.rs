use crate::settings::VERY_DARK_2;
use crate::shared_events::RefreshWorldEvent;
use bevy::app::{App, Plugin, Startup};
use bevy::core_pipeline::bloom::BloomSettings;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy_pancam::PanCam;

pub const UI_LAYER: RenderLayers = RenderLayers::layer(1);
pub const IN_GAME_LAYER: RenderLayers = RenderLayers::layer(0);

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(Startup, setup_camera_system)
      .add_systems(Update, player_controls_system)
      .insert_resource(ClearColor(VERY_DARK_2));
  }
}

#[derive(Component)]
struct WorldCamera;

#[derive(Component)]
struct UiCamera;

fn setup_camera_system(mut commands: Commands) {
  debug!("Setting up cameras");

  // Camera rending world
  commands.spawn((
    Camera2dBundle {
      camera: Camera { order: -1, ..default() },
      tonemapping: Tonemapping::TonyMcMapface,
      ..default()
    },
    WorldCamera,
    IN_GAME_LAYER,
    BloomSettings::SCREEN_BLUR,
    Name::new("Camera: In Game"),
    SpatialListener::new(10.),
    PanCam::default(),
  ));

  // Camera rendering UI
  commands.spawn((Camera2dBundle::default(), UiCamera, UI_LAYER, Name::new("Camera: UI")));
}

fn player_controls_system(
  keyboard_input: Res<ButtonInput<KeyCode>>,
  mut reset_world_event: EventWriter<RefreshWorldEvent>,
) {
  if keyboard_input.just_pressed(KeyCode::F5) {
    info!("[F5] Refreshing world...");
    reset_world_event.send(RefreshWorldEvent {});
  }
}
