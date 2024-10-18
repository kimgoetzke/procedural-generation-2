use crate::coords::point::ChunkGrid;
use crate::coords::Point;
use crate::generation::object::lib::{Connection, ObjectName};
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

  pub fn clone_and_update(&self, other_state: &Cell, where_is_other: &Connection) -> (bool, Self) {
    let currently_possible_states = self.possible_states.len();
    let mut allowed_object_names = Vec::new();
    let where_is_self_for_other = where_is_other.opposite();

    for possible_state_other in &other_state.possible_states {
      for permitted_neighbour in &possible_state_other.permitted_neighbours {
        if permitted_neighbour.0 == where_is_self_for_other {
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

    log_update(
      self,
      other_state,
      where_is_other,
      currently_possible_states,
      &mut allowed_object_names,
      where_is_self_for_other,
      &mut clone,
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
    debug!("Collapsed cg{:?} to {:?}", self.cg, rule.name);
    self.index = rule.index;
    self.is_collapsed = true;
    self.entropy = 0;
    self.possible_states = vec![rule];
  }
}

fn log_update(
  this_cell: &Cell,
  other_state: &Cell,
  where_is_other: &Connection,
  currently_possible_states: usize,
  allowed_object_names: &mut Vec<ObjectName>,
  where_is_self_for_other: Connection,
  clone: &mut Cell,
) {
  debug!(
    "Reduced possible states of cg{:?} from {} to {}",
    clone.cg,
    currently_possible_states,
    clone.possible_states.len()
  );

  if clone.possible_states.len() == 0 {
    error!(
      "Failed to find any possible states for cg{:?} ({:?}) after updating with cg{:?} ({:?})",
      clone.cg, where_is_other, other_state.cg, where_is_self_for_other
    );
  }
  if clone.possible_states.len() <= 1 {
    info!("");
    info!("START OF ANALYSIS");
    info!("At the start of this update...");
    info!(
      "- At the start of this process, there were {} possible states",
      currently_possible_states
    );
    info!(
      "- THIS cell cg{:?} was at expected to be at [{:?}] of the REFERENCE cell cg{:?})",
      this_cell.cg, where_is_other, other_state.cg
    );
    info!(
      "- The OTHER/REFERENCE cell cg{:?} was at expected to be at [{:?}] of THIS cell)",
      other_state.cg, where_is_self_for_other
    );
    info!(
      "- THIS cell had the following {} possible states:\n{:?}",
      this_cell.possible_states.len(),
      this_cell.possible_states
    );
    info!(
      "- The REFERENCE cell had the following {} possible states:\n{:?}",
      other_state.possible_states.len(),
      other_state.possible_states
    );
    info!("In this the update...");
    info!("- The allowed connections were determined to be: {:?}", allowed_object_names);
    info!(
      "- The {} new possible states were set to: {:?}",
      clone.possible_states.len(),
      clone.possible_states
    );
    info!("END OF ANALYSIS");
    info!("");
  }
  if clone.possible_states.len() == 0 {
    panic!("Failed to find any possible states while updating a cell based on a neighbour that was changed previously");
  }
}
