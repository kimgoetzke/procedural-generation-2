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

#[cfg(test)]
mod tests {
  use super::*;

  impl TerrainState {
    pub fn default(name: ObjectName, neighbours: Vec<ObjectName>) -> Self {
      Self {
        name,
        index: 1,
        weight: 10,
        permitted_neighbours: vec![
          (Connection::Top, neighbours.clone()),
          (Connection::Right, neighbours.clone()),
          (Connection::Bottom, neighbours.clone()),
          (Connection::Left, neighbours),
        ],
      }
    }
  }

  #[test]
  fn create_terrain_state_with_no_neighbours() {
    let terrain_state = TerrainState::new_with_no_neighbours(ObjectName::Land1IndividualObject1, 1, 10);
    assert_eq!(terrain_state.name, ObjectName::Land1IndividualObject1);
    assert_eq!(terrain_state.index, 1);
    assert_eq!(terrain_state.weight, 10);
    assert_eq!(terrain_state.permitted_neighbours.len(), 4);
    for (_, neighbours) in terrain_state.permitted_neighbours {
      assert_eq!(neighbours, vec![ObjectName::Empty]);
    }
  }
}
