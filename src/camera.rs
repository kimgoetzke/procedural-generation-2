use crate::constants::{CHUNK_SIZE, TILE_SIZE, WATER_BLUE};
use crate::coords::Point;
use crate::messages::{ResetCameraMessage, UpdateWorldMessage};
use crate::resources::{CurrentChunk, Settings};
use bevy::app::{App, Plugin, Startup};
use bevy::camera::visibility::RenderLayers;
use bevy::camera_controller::pan_camera::{PanCamera, PanCameraPlugin};
use bevy::input::touch::Touch;
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;

const WORLD_LAYER: RenderLayers = RenderLayers::layer(0);
const CAMERA_TRANSFORM_Z: f32 = 100000.;
const DEFAULT_PAN_SPEED: f32 = 500.0;
const SHIFT_PAN_SPEED: f32 = 2000.0;

/// Plugin to set up and manage the main game camera, including panning and touch controls.
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_plugins(PanCameraPlugin)
      .insert_resource(ClearColor(WATER_BLUE))
      .init_resource::<TouchState>()
      .add_systems(Startup, setup_camera_system)
      .add_systems(
        Update,
        (camera_movement_system, touch_camera_system, pan_cam_speed_boost_system),
      )
      .add_systems(Update, reset_camera_message);
  }
}

#[derive(Component)]
struct WorldCamera;

#[derive(Component)]
pub struct TouchCamera {
  pub pan_speed: f32,
  pub zoom_speed: f32,
  pub min_scale: f32,
  pub max_scale: f32,
}

#[derive(Resource, Default)]
struct TouchState {
  last_touch_position: Option<Vec2>,
  last_pinch_distance: Option<f32>,
  is_panning: bool,
}

fn setup_camera_system(mut commands: Commands, settings: Res<Settings>) {
  commands.spawn((
    Name::new("Camera: In Game"),
    Camera2d,
    Camera { order: 2, ..default() },
    Msaa::Off,
    Transform::from_xyz(0., 0., CAMERA_TRANSFORM_Z),
    Projection::Orthographic(OrthographicProjection {
      near: -10000.0,
      far: 1000000.0,
      scale: settings.general.camera_default_zoom,
      ..OrthographicProjection::default_3d()
    }),
    WorldCamera,
    WORLD_LAYER,
    Bloom::NATURAL,
    PanCamera {
      key_zoom_in: Some(KeyCode::PageUp),
      key_zoom_out: Some(KeyCode::PageDown),
      pan_speed: DEFAULT_PAN_SPEED,
      ..default()
    },
    TouchCamera {
      pan_speed: 1.0,
      zoom_speed: 0.003,
      min_scale: 0.15,
      max_scale: 10.0,
    },
  ));
}

fn camera_movement_system(
  camera: Query<(&Camera, &GlobalTransform)>,
  current_chunk: Res<CurrentChunk>,
  mut message: MessageWriter<UpdateWorldMessage>,
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
    message.write(UpdateWorldMessage {
      is_forced_update: false,
      tg: Point::new_tile_grid_from_world(current_world),
      w: current_world,
    });
  };
}

fn reset_camera_message(
  mut camera: Query<(&Camera, &mut Projection, &mut Transform, Option<&mut PanCamera>), With<WorldCamera>>,
  mut messages: MessageReader<ResetCameraMessage>,
  settings: Res<Settings>,
) {
  for message in messages.read() {
    let (_, mut projection, mut camera_transform, pan_camera) = camera.single_mut().expect("Failed to find camera");
    if message.reset_position {
      camera_transform.translation = Vec3::new(0., 0., CAMERA_TRANSFORM_Z);
    }
    camera_transform.rotation = Quat::IDENTITY;
    if let Projection::Orthographic(ref mut orthographic_projection) = *projection {
      orthographic_projection.scale = settings.general.camera_default_zoom;
      if let Some(mut pan_camera) = pan_camera {
        pan_camera.zoom_factor = orthographic_projection.scale;
      }
    }
    trace!("Camera position and zoom reset");
  }
}

/// A system that increases the `PanCamera`s pan speed while the `Shift` key is held. Unlike `bevy_pancam` the new Bevy
/// pan cam does not yet adjust speed based on zoom level, so we just set two fixed speeds here for now.
fn pan_cam_speed_boost_system(mut pan_cameras: Query<&mut PanCamera>, keyboard_input: Res<ButtonInput<KeyCode>>) {
  if keyboard_input.just_pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight) {
    for mut pan in pan_cameras.iter_mut() {
      pan.pan_speed = SHIFT_PAN_SPEED;
    }
    return;
  }
  if keyboard_input.just_released(KeyCode::ShiftLeft) || keyboard_input.just_released(KeyCode::ShiftRight) {
    for mut pan in pan_cameras.iter_mut() {
      pan.pan_speed = DEFAULT_PAN_SPEED;
    }
  }
}

fn touch_camera_system(
  mut camera_query: Query<(&mut Transform, &mut Projection), With<TouchCamera>>,
  touch_input: Res<Touches>,
  mut touch_state: ResMut<TouchState>,
  camera_settings: Query<&TouchCamera>,
) {
  let Ok((mut camera_transform, mut projection)) = camera_query.single_mut() else {
    return;
  };
  let Ok(settings) = camera_settings.single() else {
    return;
  };
  let touches: Vec<&Touch> = touch_input.iter().collect();
  match touches.len() {
    0 => {
      // No touches - reset state
      touch_state.last_touch_position = None;
      touch_state.last_pinch_distance = None;
      touch_state.is_panning = false;
    }
    1 => {
      // Single touch - pan camera
      let touch = touches[0];
      let current_position = touch.position();

      if let Projection::Orthographic(ref mut orthographic_projection) = *projection {
        if let Some(last_position) = touch_state.last_touch_position {
          if touch_state.is_panning {
            let delta = (last_position - current_position) * settings.pan_speed * orthographic_projection.scale;
            camera_transform.translation.x += delta.x;
            camera_transform.translation.y -= delta.y;
          }
        } else {
          touch_state.is_panning = true;
        }
      }

      touch_state.last_touch_position = Some(current_position);
      touch_state.last_pinch_distance = None;
    }
    2 => {
      // Pinch to zoom with two touches
      let touch_1 = touches[0];
      let touch_2 = touches[1];
      let position_touch_1 = touch_1.position();
      let position_touch_2 = touch_2.position();
      let current_distance = position_touch_1.distance(position_touch_2);

      if let Projection::Orthographic(ref mut orthographic_projection) = *projection {
        if let Some(last_distance) = touch_state.last_pinch_distance {
          let zoom_factor = (last_distance - current_distance) * settings.zoom_speed;
          let new_scale = (orthographic_projection.scale + zoom_factor).clamp(settings.min_scale, settings.max_scale);
          orthographic_projection.scale = new_scale;
        }

        touch_state.last_pinch_distance = Some(current_distance);

        // Bonus: Pan with the midpoint of the two touches
        let midpoint = (position_touch_1 + position_touch_2) / 2.0;
        if let Some(last_position) = touch_state.last_touch_position {
          let delta = (last_position - midpoint) * settings.pan_speed * orthographic_projection.scale;
          camera_transform.translation.x += delta.x;
          camera_transform.translation.y -= delta.y;
        }

        touch_state.last_touch_position = Some(midpoint);
        touch_state.is_panning = false;
      }
    }
    _ => {
      // Ignore more than 2 touches for now
      touch_state.last_touch_position = None;
      touch_state.last_pinch_distance = None;
      touch_state.is_panning = false;
    }
  }
}
