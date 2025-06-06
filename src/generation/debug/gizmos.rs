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
  let cam_position = camera.single().expect("Camera not found").1.translation();
  let camera_world = Point::new_world_from_world_vec2(cam_position.truncate());

  // Tile grid
  gizmos
    .grid_2d(
      current_chunk_center_world.to_vec2(),
      UVec2::new(CHUNK_SIZE as u32, CHUNK_SIZE as u32),
      Vec2::new(TILE_SIZE as f32, TILE_SIZE as f32),
      DARK,
    )
    .outer_edges();

  // Chunk grid
  gizmos
    .grid_2d(
      current_chunk_center_world.to_vec2(),
      UVec2::new(3, 3),
      Vec2::new(chunk_size, chunk_size),
      DARK,
    )
    .outer_edges();

  // Center of the current chunk and view port
  gizmos.circle_2d(current_chunk_center_world.to_vec2(), TILE_SIZE as f32, RED);

  // Line from the current world position to the center of the current chunk
  gizmos.line_2d(camera_world.to_vec2(), current_chunk_world.to_vec2(), DARK);

  // Arrow from the center of the current chunk to the current world position
  gizmos.arrow_2d(current_chunk_center_world.to_vec2(), camera_world.to_vec2(), YELLOW);
}
