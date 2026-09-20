mod buildings;
mod fields;
pub(in crate::generation) mod settlement_assets;
mod settlement_generation;

pub(crate) use fields::FieldShape;
#[cfg(test)]
pub(crate) use settlement_assets::test_settlement_resources;
pub use settlement_generation::{SettlementGenerationPlugin, place_settlement_on_grid};
