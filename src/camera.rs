use crate::settings::{CHUNK_SIZE, TILE_SIZE, VERY_DARK_2};
use crate::shared_events::RefreshWorldEvent;
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
      .add_systems(Update, player_controls_system)
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
      transform: Transform::from_xyz(
        (CHUNK_SIZE / 2 * TILE_SIZE as i32) as f32,
        (CHUNK_SIZE / 2 * TILE_SIZE as i32) as f32,
        100.,
      ),
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

fn player_controls_system(
  keyboard_input: Res<ButtonInput<KeyCode>>,
  mut reset_world_event: EventWriter<RefreshWorldEvent>,
) {
  if keyboard_input.just_pressed(KeyCode::F5) {
    info!("[F5] Refreshing world...");
    reset_world_event.send(RefreshWorldEvent {});
  }
}
