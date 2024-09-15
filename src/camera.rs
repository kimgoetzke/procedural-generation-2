use crate::constants::VERY_DARK_2;
use bevy::app::{App, Plugin, Startup};
use bevy::core_pipeline::bloom::BloomSettings;
use bevy::core_pipeline::tonemapping::Tonemapping;
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
      .insert_resource(ClearColor(VERY_DARK_2));
  }
}

#[derive(Component)]
struct WorldCamera;

fn setup_camera_system(mut commands: Commands) {
  commands.spawn((
    Camera2dBundle {
      camera: Camera { order: 2, ..default() },
      tonemapping: Tonemapping::TonyMcMapface,
      transform: Transform::from_xyz(0., 0., 100.),
      ..default()
    },
    WorldCamera,
    WORLD_LAYER,
    BloomSettings::SCREEN_BLUR,
    Name::new("Camera: In Game"),
    SpatialListener::new(10.),
    PanCam::default(),
  ));
}
