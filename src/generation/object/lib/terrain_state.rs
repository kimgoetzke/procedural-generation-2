use crate::generation::object::lib::{Connection, ObjectName};
use bevy::prelude::Reflect;

#[derive(serde::Deserialize, Debug, Clone, Reflect)]
pub struct TerrainState {
  pub name: ObjectName,
  pub index: i32,
  pub weight: i32,
  pub permitted_neighbours: Vec<(Connection, Vec<ObjectName>)>,
}
