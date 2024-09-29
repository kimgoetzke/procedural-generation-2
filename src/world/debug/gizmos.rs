use crate::constants::*;
use crate::resources::Settings;
use bevy::app::{App, Plugin};
use bevy::gizmos::AppGizmoBuilder;
use bevy::math::{UVec2, Vec2};
use bevy::prelude::{GizmoConfigGroup, Gizmos, Reflect, Res, Update};

pub struct GizmosPlugin;

impl Plugin for GizmosPlugin {
  fn build(&self, app: &mut App) {
    app.init_gizmo_group::<DebugGizmos>().add_systems(Update, draw_gizmos_system);
  }
}

#[derive(Default, Reflect, GizmoConfigGroup)]
struct DebugGizmos {}

fn draw_gizmos_system(mut gizmos: Gizmos, settings: Res<Settings>) {
  if !settings.general.draw_gizmos {
    return;
  }
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
}
