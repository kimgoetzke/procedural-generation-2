use crate::generation::world::pre_render_processor::PreRenderProcessorPlugin;
use crate::generation::world::world_generator::WorldGeneratorPlugin;
use bevy::app::{App, Plugin};

mod pre_render_processor;
mod world_generator;

pub struct WorldGenerationPlugin;

impl Plugin for WorldGenerationPlugin {
  fn build(&self, app: &mut App) {
    app.add_plugins((WorldGeneratorPlugin, PreRenderProcessorPlugin));
  }
}

pub use crate::generation::world::world_generator::*;
