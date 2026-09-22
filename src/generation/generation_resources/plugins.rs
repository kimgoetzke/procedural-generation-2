use crate::generation::generation_resources::{
  ChunkComponentIndexPlugin, GenerationResourcesCollectionPlugin, MetadataPlugin,
};
use bevy::app::{App, Plugin};

pub struct GenerationResourcesPlugins;

impl Plugin for GenerationResourcesPlugins {
  fn build(&self, app: &mut App) {
    app.add_plugins((GenerationResourcesCollectionPlugin, ChunkComponentIndexPlugin, MetadataPlugin));
  }
}
