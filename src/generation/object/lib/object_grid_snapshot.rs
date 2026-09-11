use crate::generation::object::lib::Cell;
use bevy::prelude::Reflect;

/// See [`crate::generation::object::lib::ObjectGrid`] for more info. This struct is just the minimum required for a
/// snapshot.
#[derive(Debug, Clone, Reflect)]
pub struct ObjectGridSnapshot {
  pub object_grid: Vec<Vec<Cell>>,
}
