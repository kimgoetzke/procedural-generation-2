use crate::constants::ANIMATED_OBJ_COLUMNS;
use bevy::prelude::{Component, Reflect};

#[derive(PartialEq, Reflect)]
pub enum AnimationType {
  FourFramesHalfSpeed,
  SixFramesRegularSpeed,
}

#[derive(Component, PartialEq, Reflect)]
pub struct AnimationMeshComponent {
  pub(crate) animation_type: AnimationType,
  pub(crate) columns: f32,
  pub(crate) rows: f32,
  pub(crate) tile_indices: Vec<usize>,
}

#[derive(Component, Reflect)]
pub struct AnimationSpriteComponent {
  pub(crate) animation_type: AnimationType,
  pub(crate) sprite_index: usize,
}

impl AnimationSpriteComponent {
  pub fn new(animation_type: AnimationType, sprite_index: usize) -> Self {
    Self {
      animation_type,
      sprite_index: ANIMATED_OBJ_COLUMNS as usize * sprite_index,
    }
  }
}
