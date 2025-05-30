use crate::constants::{CHUNK_SIZE, TILE_SIZE, WATER_BLUE};
use crate::coords::Point;
use crate::events::UpdateWorldEvent;
use crate::resources::CurrentChunk;
use bevy::app::{App, Plugin, Startup};
use bevy::core_pipeline::bloom::Bloom;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy_pancam::PanCam;

pub const WORLD_LAYER: RenderLayers = RenderLayers::layer(0);

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
  fn build(&self, app: &mut App) {
    app
      .insert_resource(ClearColor(WATER_BLUE))
      .add_systems(Startup, setup_camera_system)
      .add_systems(Update, camera_movement_system);
  }
}

#[derive(Component)]
struct WorldCamera;

fn setup_camera_system(mut commands: Commands) {
  commands.spawn((
    Name::new("Camera: In Game"),
    Camera2d,
    Camera { order: 2, ..default() },
    Msaa::Off,
    Transform::from_xyz(0., 0., 100000.),
    Projection::Orthographic(OrthographicProjection {
      near: -10000.0,
      far: 1000000.0,
      ..OrthographicProjection::default_3d()
    }),
    WorldCamera,
    WORLD_LAYER,
    Bloom::SCREEN_BLUR,
    SpatialListener::new(10.),
    PanCam {
      grab_buttons: vec![MouseButton::Right, MouseButton::Middle],
      speed: 600.,
      zoom_to_cursor: false,
      min_scale: 0.15,
      max_scale: 10.,
      ..default()
    },
  ));
}

fn camera_movement_system(
  camera: Query<(&Camera, &GlobalTransform)>,
  current_chunk: Res<CurrentChunk>,
  mut event: EventWriter<UpdateWorldEvent>,
) {
  let translation = camera.single().expect("Failed to find camera").1.translation();
  let current_world = Point::new_world_from_world_vec2(translation.truncate());
  let chunk_center_world = current_chunk.get_center_world();
  let distance_x = (current_world.x - chunk_center_world.x).abs();
  let distance_y = (current_world.y - chunk_center_world.y).abs();
  let trigger_distance = ((CHUNK_SIZE * TILE_SIZE as i32) / 2) + 1;
  trace!(
    "Camera moved to {:?} with distance x={:?}, y={:?} (trigger distance {})",
    current_world, distance_x, distance_y, trigger_distance
  );

  if (distance_x >= trigger_distance) || (distance_y >= trigger_distance) {
    event.write(UpdateWorldEvent {
      is_forced_update: false,
      tg: Point::new_tile_grid_from_world(current_world),
      w: current_world,
    });
  };
}
