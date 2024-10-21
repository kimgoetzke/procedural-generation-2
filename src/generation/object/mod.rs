mod components;
pub(crate) mod lib;
mod object_generator;
mod wfc;

use crate::generation::object::components::ObjectGenerationComponentsPlugin;
use bevy::app::{App, Plugin};

pub struct ObjectGenerationPlugin;

impl Plugin for ObjectGenerationPlugin {
  fn build(&self, app: &mut App) {
    app.add_plugins((ObjectGeneratorPlugin, ObjectGenerationComponentsPlugin));
  }
}

pub use crate::generation::object::object_generator::generate;
use crate::generation::object::object_generator::ObjectGeneratorPlugin;
