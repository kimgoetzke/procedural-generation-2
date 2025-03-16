use crate::generation::world::metadata_generator::MetadataGeneratorPlugin;
use crate::generation::world::post_processor::PostProcessorPlugin;
use crate::generation::world::world_generator::WorldGeneratorPlugin;
use bevy::app::{App, Plugin};

mod metadata_generator;
mod post_processor;
mod world_generator;

pub struct WorldGenerationPlugin;

impl Plugin for WorldGenerationPlugin {
  fn build(&self, app: &mut App) {
    app.add_plugins((MetadataGeneratorPlugin, WorldGeneratorPlugin, PostProcessorPlugin));
  }
}

pub use crate::generation::world::world_generator::{generate_chunks, spawn_layer_meshes, spawn_chunk};
