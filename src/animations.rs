#![allow(unused_imports, dead_code)]

use crate::components::{AnimationMeshComponent, AnimationSpriteComponent};
use bevy::app::{App, Plugin};
use bevy::asset::Assets;
use bevy::prelude::{Mesh, Mesh2d, Query, Res, ResMut, Sprite, Time, Update};
use bevy::render::mesh::VertexAttributeValues;

pub struct AnimationsPlugin;

impl Plugin for AnimationsPlugin {
  fn build(&self, app: &mut App) {
    app
      // .add_systems(Update, sprite_animation_system) // Use once animated objects are used
      .add_systems(Update, animate_tile_meshes);
  }
}

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

fn animate_tile_meshes(
  time: Res<Time>,
  mut meshes: ResMut<Assets<Mesh>>,
  mut query: Query<(&mut AnimationMeshComponent, &Mesh2d)>,
) {
  for (mut anim_mesh_component, mesh_2d) in &mut query {
    anim_mesh_component.timer.tick(time.delta());
    if anim_mesh_component.timer.just_finished() {
      anim_mesh_component.current_frame = (anim_mesh_component.current_frame + 1) % anim_mesh_component.frame_count;
      if let Some(mesh) = meshes.get_mut(mesh_2d) {
        if let Some(uv_attribute) = mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0) {
          if let VertexAttributeValues::Float32x2(uvs) = uv_attribute {
            let mut tile_index = 0;
            for i in 0..uvs.len() / 4 {
              let base_sprite_index = anim_mesh_component.tile_indices[tile_index];
              let frame_offset = anim_mesh_component.current_frame;
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
    }
  }
}
