mod chunk_component_index;
mod generation_resources_collection;
mod metadata;
mod object_asset_initialisation;
mod plugins;
pub(in crate::generation) mod settlement_asset_initialisation;
mod terrain_asset_initialisation;
mod terrain_state_initialisation;
mod terrain_state_validation;

pub use chunk_component_index::ChunkComponentIndexPlugin;
pub use generation_resources_collection::GenerationResourcesCollectionPlugin;
pub use metadata::MetadataPlugin;
pub use plugins::GenerationResourcesPlugins;
