mod object_generator;

use bevy::app::{App, Plugin};

pub struct ObjectGenerationPlugin;

impl Plugin for ObjectGenerationPlugin {
  fn build(&self, app: &mut App) {
    app.add_plugins(ObjectGeneratorPlugin);
  }
}

pub use crate::generation::object::object_generator::generate;
use crate::generation::object::object_generator::ObjectGeneratorPlugin;
