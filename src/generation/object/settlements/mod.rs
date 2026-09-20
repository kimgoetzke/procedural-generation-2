mod buildings;
mod fields;
mod settlement_generation;

pub(crate) use fields::FieldShape;
pub use settlement_generation::{SettlementGenerationPlugin, place_settlement_on_grid};
