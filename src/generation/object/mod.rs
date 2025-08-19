pub(crate) mod lib;
mod object_generator;
pub mod path;
pub mod wfc;

use crate::generation::object::object_generator::ObjectGeneratorPlugin;
use crate::generation::object::path::PathGenerationPlugin;
use bevy::app::{App, Plugin};

pub struct ObjectGenerationPlugin;

impl Plugin for ObjectGenerationPlugin {
  fn build(&self, app: &mut App) {
    app.add_plugins((ObjectGeneratorPlugin, PathGenerationPlugin));
  }
}

pub use crate::generation::object::object_generator::*;
