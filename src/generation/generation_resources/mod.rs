mod chunk_component_index;
mod generation_resources;
mod generation_resources_collection;
mod metadata;
pub(in crate::generation) mod settlement_asset_initialisation;

pub use chunk_component_index::ChunkComponentIndexPlugin;
pub use generation_resources::GenerationResourcesPlugin;
pub use generation_resources_collection::GenerationResourcesCollectionPlugin;
pub use metadata::MetadataPlugin;
