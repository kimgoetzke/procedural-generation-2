use crate::generation::generation_resources::{
  ChunkComponentIndexPlugin, GenerationResourcesCollectionPlugin, MetadataPlugin,
};
use bevy::app::{App, Plugin};

pub struct GenerationResourcesPlugin;

impl Plugin for GenerationResourcesPlugin {
  fn build(&self, app: &mut App) {
    app.add_plugins((GenerationResourcesCollectionPlugin, ChunkComponentIndexPlugin, MetadataPlugin));
  }
}
