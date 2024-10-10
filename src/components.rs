use bevy::prelude::{Component, Deref, DerefMut, Timer};

#[derive(Component)]
pub struct AnimationComponent {
  pub(crate) index_first: usize,
  pub(crate) index_last: usize,
  pub(crate) timer: AnimationTimer,
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(pub Timer);
