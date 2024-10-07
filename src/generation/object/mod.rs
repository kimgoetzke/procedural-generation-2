mod object_generator;

use bevy::app::{App, Plugin};

pub struct ObjectGenerationPlugin;

impl Plugin for ObjectGenerationPlugin {
  fn build(&self, _app: &mut App) {}
}

pub use crate::generation::object::object_generator::generate;
