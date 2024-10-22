use crate::coords::point::ChunkGrid;
use crate::coords::Point;
use crate::generation::lib::TerrainType;
use crate::generation::object::lib::{Connection, ObjectName, PropagationFailure};
use crate::generation::resources::TileState;
use bevy::log::*;
use bevy::prelude::Reflect;
use rand::prelude::StdRng;
use rand::Rng;

#[derive(Debug, Clone, Reflect)]
pub struct Cell {
  pub cg: Point<ChunkGrid>,
  pub is_collapsed: bool,
  is_initialised: bool,
  pub terrain: TerrainType,
  pub entropy: usize,
  pub possible_states: Vec<TileState>,
  pub index: i32,
}

impl Cell {
  pub fn new(x: i32, y: i32) -> Self {
    Cell {
      cg: Point::new_chunk_grid(x, y),
      is_collapsed: false,
      is_initialised: false,
      terrain: TerrainType::Any,
      entropy: usize::MAX,
      possible_states: vec![],
      index: -1,
    }
  }

  pub fn initialise(&mut self, terrain_type: TerrainType, states: &Vec<TileState>) {
    if self.is_initialised {
      panic!("Attempting to initialise a cell that already has been initialised");
    }
    self.terrain = terrain_type;
    self.possible_states = states.clone();
    self.entropy = self.possible_states.len();
    self.is_initialised = true;
  }

  pub fn clone_and_reduce(
    &self,
    reference_cell: &Cell,
    where_is_reference: &Connection,
  ) -> Result<(bool, Self), PropagationFailure> {
    let count_currently_possible_states = self.possible_states.len();
    let where_is_self_for_reference = where_is_reference.opposite();
    let permitted_state_names = get_permitted_new_states(&reference_cell, where_is_self_for_reference);

    let mut updated_possible_states = Vec::new();
    for possible_state_self in &self.possible_states {
      if permitted_state_names.contains(&possible_state_self.name) {
        updated_possible_states.push(possible_state_self.clone());
      };
    }

    let mut clone = self.clone();
    clone.possible_states = updated_possible_states;
    clone.entropy = self.possible_states.len();
    log(
      true,
      reference_cell,
      where_is_reference,
      where_is_self_for_reference,
      self,
      count_currently_possible_states,
      &mut clone,
      &permitted_state_names,
    );

    match clone.possible_states.len() {
      0 => Err(PropagationFailure {}),
      _ => Ok((count_currently_possible_states != clone.possible_states.len(), clone)),
    }
  }

  // TODO: Add weights to the rules at some point
  pub fn collapse(&mut self, rng: &mut StdRng) {
    let rule = self.possible_states.remove(rng.gen_range(0..self.possible_states.len()));
    trace!("Collapsed cg{:?} to {:?}", self.cg, rule.name);
    self.index = rule.index;
    self.is_collapsed = true;
    self.entropy = 0;
    self.possible_states = vec![rule];
  }

  pub fn verify(&self, reference_cell: &Cell, where_is_reference: &Connection) -> Result<(), PropagationFailure> {
    let where_is_self_for_reference = where_is_reference.opposite();
    let permitted_state_names = get_permitted_new_states(&reference_cell, where_is_self_for_reference);

    if !permitted_state_names.contains(&self.possible_states[0].name) {
      log(
        false,
        reference_cell,
        where_is_reference,
        where_is_self_for_reference,
        self,
        self.possible_states.len(),
        &mut self.clone(),
        &permitted_state_names,
      );
      Err(PropagationFailure {})
    } else {
      Ok(())
    }
  }
}

fn get_permitted_new_states(reference_cell: &Cell, where_is_self_for_reference: Connection) -> Vec<ObjectName> {
  reference_cell
    .possible_states
    .iter()
    .flat_map(|possible_state_reference| {
      possible_state_reference
        .permitted_neighbours
        .iter()
        .filter(|(connection, _)| *connection == where_is_self_for_reference)
        .flat_map(|(_, names)| names.iter().map(|(name, _)| name.clone()))
    })
    .collect()
}

fn log(
  is_update: bool,
  reference_cell: &Cell,
  reference_connection: &Connection,
  cell_connection: Connection,
  old_cell: &Cell,
  old_possible_states_count: usize,
  new_cell: &mut Cell,
  new_permitted_states: &Vec<ObjectName>,
) {
  if old_possible_states_count != new_cell.possible_states.len() && is_update {
    trace!(
      "Reduced possible states of cg{:?} from {} to {}",
      new_cell.cg,
      old_possible_states_count,
      new_cell.possible_states.len()
    );
  }

  if new_cell.possible_states.len() == 0 {
    trace!(
      "Failed to find any possible states for cg{:?} ({:?} {:?}) during {} with cg{:?} ({:?})",
      new_cell.cg,
      old_cell.terrain,
      reference_connection,
      if is_update { "update" } else { "verification" },
      reference_cell.cg,
      reference_cell.terrain,
    );
    trace!(
      "Summary of the {} process for cg{:?}:",
      if is_update { "UPDATE" } else { "VERIFICATION" },
      old_cell.cg
    );
    trace!(
      "- THIS cell is at cg{:?} which is at the [{:?}] of the reference cell",
      old_cell.cg,
      reference_connection
    );
    trace!(
      "- The REFERENCE cell is at cg{:?} which is at the [{:?}] of this cell)",
      reference_cell.cg,
      cell_connection
    );
    trace!(
      "- The REFERENCE cell has the following {} possible states: {:?}",
      reference_cell.possible_states.len(),
      reference_cell
        .possible_states
        .iter()
        .map(|s| s.name)
        .collect::<Vec<ObjectName>>()
    );
    trace!("- The permitted new states were determined to be: {:?}", new_permitted_states);
    trace!(
      "- At the start of the update process, THIS cell had {:?} possible states: {:?}",
      old_possible_states_count,
      old_cell.possible_states.iter().map(|s| s.name).collect::<Vec<ObjectName>>()
    );
    trace!(
      "- The {} new possible states for THIS cell are: {:?}",
      new_cell.possible_states.len(),
      new_cell.possible_states.iter().map(|s| s.name).collect::<Vec<ObjectName>>()
    );
    trace!("");
  }
}
