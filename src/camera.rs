use crate::constants::WATER_BLUE;
use bevy::app::{App, Plugin, Startup};
use bevy::core_pipeline::bloom::BloomSettings;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy_pancam::PanCam;

pub const WORLD_LAYER: RenderLayers = RenderLayers::layer(0);

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(Startup, setup_camera_system)
      .insert_resource(Msaa::Off)
      .insert_resource(ClearColor(WATER_BLUE));
  }
}

#[derive(Component)]
struct WorldCamera;

fn setup_camera_system(mut commands: Commands) {
  commands.spawn((
    Camera2dBundle {
      camera: Camera { order: 2, ..default() },
      transform: Transform::from_xyz(0., 0., 100.),
      ..default()
    },
    WorldCamera,
    WORLD_LAYER,
    BloomSettings::SCREEN_BLUR,
    Name::new("Camera: In Game"),
    SpatialListener::new(10.),
    PanCam {
      grab_buttons: vec![MouseButton::Right, MouseButton::Middle],
      speed: 400.,
      zoom_to_cursor: false,
      min_scale: 0.15,
      max_scale: 5.,
      ..default()
    },
  ));
}
