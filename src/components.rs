use bevy::prelude::{Component, Deref, DerefMut, Timer};

#[derive(Component)]
pub struct AnimationSpriteComponent {
  pub(crate) index_first: usize,
  pub(crate) index_last: usize,
  pub(crate) timer: AnimationTimer,
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(pub Timer);

#[derive(PartialEq)]
pub enum AnimationType {
  FourFramesDefaultSpeed,
  FourFramesDoubleSpeed,
}

#[derive(Component, PartialEq)]
pub struct AnimationMeshComponent {
  pub(crate) animation_type: AnimationType,
  pub(crate) columns: f32,
  pub(crate) rows: f32,
  pub(crate) tile_indices: Vec<usize>,
}
