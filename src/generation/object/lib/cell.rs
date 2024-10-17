use crate::coords::point::ChunkGrid;
use crate::coords::Point;
use crate::generation::object::lib::Connection;
use crate::generation::resources::{RuleSet, TileState};
use bevy::log::*;
use rand::prelude::StdRng;
use rand::Rng;

#[derive(Debug, Clone)]
pub struct Cell {
  pub cg: Point<ChunkGrid>,
  pub is_collapsed: bool,
  pub entropy: usize,
  pub possible_states: Vec<TileState>,
  pub index: i32,
}

impl Cell {
  pub fn new(x: i32, y: i32, rule_set: &RuleSet) -> Self {
    Cell {
      cg: Point::new_chunk_grid(x, y),
      is_collapsed: false,
      entropy: rule_set.states.len(),
      possible_states: rule_set.states.clone(),
      index: -1,
    }
  }

  pub fn clone_and_update(&self, other_state: &Cell, connection_to_self: &Connection) -> (bool, Self) {
    let currently_possible_states = self.possible_states.len();
    let mut allowed_object_names = Vec::new();
    let connection_to_other = connection_to_self.opposite();

    for possible_state_other in &other_state.possible_states {
      for permitted_neighbour in &possible_state_other.permitted_neighbours {
        if permitted_neighbour.0 == connection_to_other {
          let object_name = permitted_neighbour
            .1
            .iter()
            .map(|(name, _)| name)
            .next()
            .expect("Failed to find object name")
            .clone();
          allowed_object_names.push(object_name);
        }
      }
    }

    let mut new_possible_states = Vec::new();
    for possible_state_self in &self.possible_states {
      if allowed_object_names.contains(&possible_state_self.name) {
        new_possible_states.push(possible_state_self.clone());
      };
    }

    let mut clone = self.clone();
    clone.possible_states = new_possible_states;
    clone.entropy = self.possible_states.len();

    debug!(
      "Reduced possible states of cg{:?} from {} to {}",
      clone.cg,
      currently_possible_states,
      clone.possible_states.len()
    );

    (currently_possible_states != clone.possible_states.len(), clone)
  }

  pub fn collapse_to_empty(&mut self) -> &Self {
    let rule = self.possible_states.get(0).unwrap().clone();
    self.index = rule.index;
    self.possible_states = vec![rule];
    self.is_collapsed = true;
    self.entropy = 0;

    self
  }

  // TODO: Add weights to the rules at some point
  pub fn collapse(&mut self, rng: &mut StdRng) {
    let rule = self.possible_states.remove(rng.gen_range(0..self.possible_states.len()));
    self.index = rule.index;
    self.is_collapsed = true;
    self.entropy = 0;
    self.possible_states = vec![rule];
  }
}
