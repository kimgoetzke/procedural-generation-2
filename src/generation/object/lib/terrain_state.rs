use crate::generation::object::lib::{Connection, ObjectName};
use bevy::prelude::Reflect;

#[derive(serde::Deserialize, Debug, Clone, Reflect)]
pub struct TerrainState {
  pub name: ObjectName,
  pub index: i32,
  pub weight: i32,
  pub permitted_neighbours: Vec<(Connection, Vec<ObjectName>)>,
}

impl TerrainState {
  pub fn new_with_no_neighbours(name: ObjectName, index: i32, weight: i32) -> Self {
    Self {
      name,
      index,
      weight,
      permitted_neighbours: vec![
        (Connection::Top, vec![ObjectName::Empty]),
        (Connection::Right, vec![ObjectName::Empty]),
        (Connection::Bottom, vec![ObjectName::Empty]),
        (Connection::Left, vec![ObjectName::Empty]),
      ],
    }
  }
}
