pub(crate) mod lib;
mod object_generator;
mod wfc;

use crate::generation::object::object_generator::ObjectGeneratorPlugin;
use bevy::app::{App, Plugin};

pub struct ObjectGenerationPlugin;

impl Plugin for ObjectGenerationPlugin {
  fn build(&self, app: &mut App) {
    app.add_plugins(ObjectGeneratorPlugin);
  }
}

pub use crate::generation::object::object_generator::{generate_object_data, schedule_spawning_objects};
