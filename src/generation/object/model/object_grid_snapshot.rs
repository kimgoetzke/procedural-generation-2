use crate::generation::object::model::Cell;
use bevy::prelude::Reflect;

/// See [`crate::generation::object::model::ObjectGrid`] for more info. This struct is just the minimum required for a
/// snapshot.
#[derive(Debug, Clone, Reflect)]
pub struct ObjectGridSnapshot {
  pub object_grid: Vec<Vec<Cell>>,
}
