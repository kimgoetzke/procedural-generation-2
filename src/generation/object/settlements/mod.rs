mod buildings;
mod fields;
mod settlement_generation;

#[cfg(test)]
pub(crate) use crate::generation::generation_resources::settlement_asset_initialisation::test_settlement_resources;
pub(crate) use fields::FieldShape;
pub use settlement_generation::{SettlementGenerationPlugin, place_settlement_on_grid};
