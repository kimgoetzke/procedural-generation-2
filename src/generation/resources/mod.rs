mod chunk_component_index;
mod generation_resources;
mod generation_resources_collection;
mod metadata;
mod settlement_assets;

pub use chunk_component_index::*;
pub use generation_resources::GenerationResourcesPlugin;
pub use metadata::*;
#[cfg(test)]
pub(crate) use settlement_assets::test_settlement_resources;
