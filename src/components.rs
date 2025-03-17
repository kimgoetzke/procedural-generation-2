use bevy::prelude::{Component, Deref, DerefMut, Timer};

#[derive(Component)]
pub struct AnimationSpriteComponent {
  pub(crate) index_first: usize,
  pub(crate) index_last: usize,
  pub(crate) timer: AnimationTimer,
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(pub Timer);

#[derive(Component)]
pub struct AnimationMeshComponent {
  pub(crate) timer: AnimationTimer,
  pub(crate) frame_count: usize,
  pub(crate) current_frame: usize,
  pub(crate) columns: f32,
  pub(crate) rows: f32,
  pub(crate) tile_indices: Vec<usize>,
}
