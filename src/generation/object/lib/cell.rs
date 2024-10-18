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

  pub fn clone_and_reduce(&self, reference_cell: &Cell, where_is_reference: &Connection) -> (bool, Self) {
    let currently_possible_states = self.possible_states.len();
    let mut allowed_object_names: Vec<ObjectName> = Vec::new();
    let where_is_self_for_reference = where_is_reference.opposite();

    for possible_state_other in &reference_cell.possible_states {
      for permitted_neighbour in &possible_state_other.permitted_neighbours {
        if permitted_neighbour.0 == where_is_self_for_reference {
          for (name, _) in &permitted_neighbour.1 {
            allowed_object_names.push(name.clone());
          }
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
      reference_cell,
      where_is_reference,
      where_is_self_for_reference,
      self,
      currently_possible_states,
      &mut clone,
      &mut allowed_object_names,
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
  reference_cell: &Cell,
  reference_connection: &Connection,
  cell_connection: Connection,
  old_cell: &Cell,
  old_possible_states_count: usize,
  new_cell: &mut Cell,
  new_possible_states: &mut Vec<ObjectName>,
) {
  if old_possible_states_count != new_cell.possible_states.len() {
    debug!(
      "Reduced possible states of cg{:?} from {} to {}",
      new_cell.cg,
      old_possible_states_count,
      new_cell.possible_states.len()
    );
  }

  if new_cell.possible_states.len() == 0 {
    error!(
      "Failed to find any possible states for cg{:?} ({:?}) during update with cg{:?}",
      new_cell.cg, reference_connection, reference_cell.cg
    );
    debug!("Summary of the update process for cg{:?}:", old_cell.cg);
    debug!(
      "- THIS cell is at cg{:?} which is at the [{:?}] of the reference cell",
      old_cell.cg, reference_connection
    );
    debug!(
      "- The REFERENCE cell is at cg{:?} which is at the [{:?}] of this cell)",
      reference_cell.cg, cell_connection
    );
    debug!(
      "- The REFERENCE cell has the following {} possible states: {:?}",
      reference_cell.possible_states.len(),
      reference_cell
        .possible_states
        .iter()
        .map(|s| s.name)
        .collect::<Vec<ObjectName>>()
    );
    debug!("- The permitted new states were determined to be: {:?}", new_possible_states);
    debug!(
      "- At the start of the update process, THIS cell had {:?} possible states: {:?}",
      old_possible_states_count,
      old_cell.possible_states.iter().map(|s| s.name).collect::<Vec<ObjectName>>()
    );
    debug!(
      "- The {} new possible states for THIS cell are: {:?}",
      new_cell.possible_states.len(),
      new_cell.possible_states.iter().map(|s| s.name).collect::<Vec<ObjectName>>()
    );
    debug!("");
  }

  if new_cell.possible_states.len() == 0 {
    panic!("Failed to find any possible states while updating a cell based on a neighbour that was changed previously");
  }
}
