use crate::components::{AnimationMeshComponent, AnimationSpriteComponent, AnimationType};
use crate::constants::{
  DEFAULT_ANIMATED_TILE_SET_COLUMNS, DEFAULT_ANIMATION_FRAME_DURATION, ENHANCED_ANIMATED_TILE_SET_COLUMNS,
};
use bevy::app::{App, Plugin};
use bevy::asset::Assets;
use bevy::prelude::{Mesh, Mesh2d, Mut, Query, Res, ResMut, Resource, Sprite, Time, Timer, TimerMode, Update};
use bevy::render::mesh::VertexAttributeValues;

pub struct AnimationsPlugin;

impl Plugin for AnimationsPlugin {
  fn build(&self, app: &mut App) {
    app
      .insert_resource(GlobalAnimationState::new())
      .register_type::<AnimationMeshComponent>()
      // .add_systems(Update, sprite_animation_system) // Use once animated objects are used
      .add_systems(Update, tile_mesh_animation_system);
  }
}

struct AnimationTypeState {
  animation_type: AnimationType,
  timer: Timer,
  current_frame: u32,
  total_frames: u32,
}

/// Stores the animation state for all types of animations in the application. States are global to allow cross-chunk
/// animations to stay in sync. Different animation types can be registered here with their own timers and frame counts.
/// To use them, spawn an entity with the appropriate [`AnimationMeshComponent`].
#[derive(Resource)]
struct GlobalAnimationState {
  types: Vec<AnimationTypeState>,
}

impl GlobalAnimationState {
  fn new() -> Self {
    Self {
      types: vec![
        AnimationTypeState {
          animation_type: AnimationType::FourFramesHalfSpeed,
          timer: Timer::from_seconds(DEFAULT_ANIMATION_FRAME_DURATION * 2., TimerMode::Repeating),
          current_frame: 0,
          total_frames: DEFAULT_ANIMATED_TILE_SET_COLUMNS,
        },
        AnimationTypeState {
          animation_type: AnimationType::SixFramesRegularSpeed,
          timer: Timer::from_seconds(DEFAULT_ANIMATION_FRAME_DURATION, TimerMode::Repeating),
          current_frame: 0,
          total_frames: ENHANCED_ANIMATED_TILE_SET_COLUMNS,
        },
      ],
    }
  }
}

#[allow(unused)]
fn sprite_animation_system(time: Res<Time>, mut query: Query<(&mut AnimationSpriteComponent, &mut Sprite)>) {
  for (mut ac, mut sprite) in &mut query {
    ac.timer.tick(time.delta());
    if ac.timer.just_finished() {
      if let Some(atlas) = &mut sprite.texture_atlas {
        atlas.index = if atlas.index >= ac.index_last {
          ac.index_first
        } else {
          atlas.index + 1
        };
      }
    }
  }
}

fn tile_mesh_animation_system(
  time: Res<Time>,
  mut animation_states: ResMut<GlobalAnimationState>,
  mut meshes: ResMut<Assets<Mesh>>,
  mut query: Query<(&mut AnimationMeshComponent, &Mesh2d)>,
) {
  for state in &mut animation_states.types {
    state.timer.tick(time.delta());
    if state.timer.just_finished() {
      state.current_frame = (state.current_frame + 1) % state.total_frames;

      for (animated_mesh_component, mesh_2d) in &mut query {
        if state.animation_type != animated_mesh_component.animation_type {
          continue;
        }
        if let Some(mesh) = meshes.get_mut(mesh_2d) {
          update_mesh_uvs(state, animated_mesh_component, mesh);
        }
      }
    }
  }
}

fn update_mesh_uvs(
  animation_state: &mut AnimationTypeState,
  anim_mesh_component: Mut<AnimationMeshComponent>,
  mesh: &mut Mesh,
) {
  if let Some(uv_attribute) = mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0) {
    if let VertexAttributeValues::Float32x2(uvs) = uv_attribute {
      let mut tile_index = 0;
      for i in 0..uvs.len() / 4 {
        let base_sprite_index = anim_mesh_component.tile_indices[tile_index];
        let frame_offset = animation_state.current_frame as usize;
        let sprite_index = base_sprite_index + frame_offset;
        let sprite_col = sprite_index as f32 % anim_mesh_component.columns;
        let sprite_row = (sprite_index as f32 / anim_mesh_component.columns).floor();

        let u_start = sprite_col / anim_mesh_component.columns;
        let u_end = (sprite_col + 1.0) / anim_mesh_component.columns;
        let v_start = sprite_row / anim_mesh_component.rows;
        let v_end = (sprite_row + 1.0) / anim_mesh_component.rows;

        let vertex_base = i * 4;
        uvs[vertex_base] = [u_start, v_start]; // Top-left
        uvs[vertex_base + 1] = [u_end, v_start]; // Top-right
        uvs[vertex_base + 2] = [u_end, v_end]; // Bottom-right
        uvs[vertex_base + 3] = [u_start, v_end]; // Bottom-left

        tile_index += 1;

        if tile_index >= anim_mesh_component.tile_indices.len() {
          break;
        }
      }
    }
  }
}
