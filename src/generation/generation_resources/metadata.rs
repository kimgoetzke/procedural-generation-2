use crate::generation::model::{BiomeMetadata, Metadata};
use bevy::app::{App, Plugin};

pub struct MetadataPlugin;

impl Plugin for MetadataPlugin {
  fn build(&self, app: &mut App) {
    app
      .init_resource::<Metadata>()
      .register_type::<Metadata>()
      .register_type::<BiomeMetadata>();
  }
}
