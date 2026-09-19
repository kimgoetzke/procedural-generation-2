use crate::generation::object::decoration::WfcPlugin;
use crate::generation::object::object_generator::ObjectGeneratorPlugin;
use crate::generation::object::path::PathGenerationPlugin;
use crate::generation::object::settlements::SettlementGenerationPlugin;
use bevy::app::{App, Plugin};

pub struct ObjectGenerationPlugin;

impl Plugin for ObjectGenerationPlugin {
  fn build(&self, app: &mut App) {
    app.add_plugins((
      ObjectGeneratorPlugin,
      PathGenerationPlugin,
      SettlementGenerationPlugin,
      WfcPlugin,
    ));
  }
}
