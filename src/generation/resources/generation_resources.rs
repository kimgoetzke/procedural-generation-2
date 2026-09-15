use crate::generation::resources::chunk_component_index::ChunkComponentIndexPlugin;
use crate::generation::resources::generation_resources_collection::GenerationResourcesCollectionPlugin;
use crate::generation::resources::metadata::MetadataPlugin;
use bevy::app::{App, Plugin};

pub struct GenerationResourcesPlugin;

impl Plugin for GenerationResourcesPlugin {
  fn build(&self, app: &mut App) {
    app.add_plugins((GenerationResourcesCollectionPlugin, ChunkComponentIndexPlugin, MetadataPlugin));
  }
}
