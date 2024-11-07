use crate::generation::world::metadata_generator::MetadataGeneratorPlugin;
use crate::generation::world::pre_render_processor::PreRenderProcessorPlugin;
use crate::generation::world::world_generator::WorldGeneratorPlugin;
use bevy::app::{App, Plugin};

mod metadata_generator;
mod pre_render_processor;
mod world_generator;

pub struct WorldGenerationPlugin;

impl Plugin for WorldGenerationPlugin {
  fn build(&self, app: &mut App) {
    app.add_plugins((MetadataGeneratorPlugin, WorldGeneratorPlugin, PreRenderProcessorPlugin));
  }
}

pub use crate::generation::world::world_generator::{generate_chunks, schedule_tile_spawning_tasks, spawn_chunk};
