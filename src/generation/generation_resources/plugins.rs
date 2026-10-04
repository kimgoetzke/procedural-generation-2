use crate::generation::generation_resources::current_chunk::CurrentChunkPlugin;
use crate::generation::generation_resources::{ChunkComponentIndexPlugin, GenerationResourcesPlugin, MetadataPlugin};
use bevy::app::{App, Plugin};

pub struct GenerationResourcesPlugins;

impl Plugin for GenerationResourcesPlugins {
  fn build(&self, app: &mut App) {
    app.add_plugins((
      GenerationResourcesPlugin,
      ChunkComponentIndexPlugin,
      CurrentChunkPlugin,
      MetadataPlugin,
    ));
  }
}
