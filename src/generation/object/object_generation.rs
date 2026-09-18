use crate::generation::object::object_generator::ObjectGeneratorPlugin;
use crate::generation::object::path::PathGenerationPlugin;
use crate::generation::object::structures::StructureGenerationPlugin;
use crate::generation::object::wfc::WfcPlugin;
use bevy::app::{App, Plugin};

pub struct ObjectGenerationPlugin;

impl Plugin for ObjectGenerationPlugin {
  fn build(&self, app: &mut App) {
    app.add_plugins((
      ObjectGeneratorPlugin,
      PathGenerationPlugin,
      StructureGenerationPlugin,
      WfcPlugin,
    ));
  }
}
