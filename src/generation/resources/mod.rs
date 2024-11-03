mod chunk_component_index;
mod generation_resources_collection;
mod metadata_component_index;

use crate::generation::resources::chunk_component_index::ChunkComponentIndexPlugin;
use crate::generation::resources::generation_resources_collection::GenerationResourcesCollectionPlugin;
use bevy::app::{App, Plugin};

pub struct GenerationResourcesPlugin;

impl Plugin for GenerationResourcesPlugin {
  fn build(&self, app: &mut App) {
    app.add_plugins((
      GenerationResourcesCollectionPlugin,
      ChunkComponentIndexPlugin,
      MetadataComponentIndexPlugin,
    ));
  }
}

pub use crate::generation::resources::chunk_component_index::*;
pub use crate::generation::resources::generation_resources_collection::*;
pub use crate::generation::resources::metadata_component_index::*;
