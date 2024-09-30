use crate::constants::*;
use crate::coords::Point;
use crate::resources::{CurrentChunk, Settings};
use bevy::app::{App, Plugin};
use bevy::gizmos::AppGizmoBuilder;
use bevy::math::{UVec2, Vec2};
use bevy::prelude::*;

pub struct GizmosPlugin;

impl Plugin for GizmosPlugin {
  fn build(&self, app: &mut App) {
    app.init_gizmo_group::<DebugGizmos>().add_systems(Update, draw_gizmos_system);
  }
}

#[derive(Default, Reflect, GizmoConfigGroup)]
struct DebugGizmos {}

fn draw_gizmos_system(
  mut gizmos: Gizmos,
  settings: Res<Settings>,
  current_chunk: Res<CurrentChunk>,
  camera: Query<(&Camera, &GlobalTransform)>,
) {
  if !settings.general.draw_gizmos {
    return;
  }

  let current_chunk_center_world = current_chunk.get_center_world();
  let current_chunk_world = current_chunk.get_world();
  let chunk_size = TILE_SIZE as f32 * CHUNK_SIZE as f32;
  let cam_position = camera.single().1.translation();
  let camera_world = Point::new_world_from_world_vec2(cam_position.truncate());

  // Tile grid for the origin chunk
  gizmos
    .grid_2d(
      Vec2 {
        x: -(TILE_SIZE as f32 / 2.),
        y: TILE_SIZE as f32 / 2.,
      },
      0.0,
      UVec2::new(CHUNK_SIZE as u32, CHUNK_SIZE as u32),
      Vec2::new(32., 32.),
      DARK_1,
    )
    .outer_edges();

  // Center tile in the current chunk
  gizmos.rect_2d(
    Vec2::new(current_chunk_center_world.x as f32, current_chunk_center_world.y as f32),
    0.,
    Vec2::new(TILE_SIZE as f32, TILE_SIZE as f32),
    GREEN,
  );

  // Current chunk
  gizmos.rect_2d(
    Vec2::new(current_chunk_center_world.x as f32, current_chunk_center_world.y as f32),
    0.,
    Vec2::new(chunk_size, chunk_size),
    ORANGE,
  );

  // Current chunk x 2
  gizmos.rect_2d(
    Vec2::new(current_chunk_center_world.x as f32, current_chunk_center_world.y as f32),
    0.,
    Vec2::new(chunk_size * 2., chunk_size * 2.),
    RED,
  );

  // Line from the current world position to the center of the current chunk
  gizmos.line_2d(camera_world.to_vec2(), current_chunk_world.to_vec2(), GREEN);

  // Arrow from the center of the current chunk to the current world position
  gizmos.arrow_2d(current_chunk_center_world.to_vec2(), camera_world.to_vec2(), YELLOW);
}
