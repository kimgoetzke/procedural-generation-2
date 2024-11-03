use crate::coords::point::InternalGrid;
use crate::coords::Point;
use crate::generation::lib::{TerrainType, TileType};
use crate::generation::object::lib::{Connection, ObjectName};
use crate::generation::resources::TerrainState;
use bevy::log::*;
use bevy::prelude::Reflect;
use rand::prelude::StdRng;
use rand::Rng;

pub struct PropagationFailure {}

/// A `Cell` is a "placeholder" for an object. It is used in the `ObjectGrid`. This struct is used to represent a cell in
/// the grid that can be collapsed to a single state. Once all `Cell`s in an `ObjectGrid` have been collapsed, they
/// will be converted to `ObjectData`s which are then used to spawn object sprites in the world. A `Cell` is
/// indirectly linked to an underlying `Tile` through its `TerrainType` and  `TileType` fields.
#[derive(Debug, Clone, Reflect)]
pub struct Cell {
  pub ig: Point<InternalGrid>,
  pub is_collapsed: bool,
  is_initialised: bool,
  is_being_monitored: bool,
  pub terrain: TerrainType,
  pub tile_type: TileType,
  pub entropy: usize,
  pub possible_states: Vec<TerrainState>,
  pub index: i32,
}

impl Cell {
  pub fn new(x: i32, y: i32) -> Self {
    Cell {
      ig: Point::new_internal_grid(x, y),
      is_collapsed: false,
      is_initialised: false,
      is_being_monitored: false,
      terrain: TerrainType::Any,
      tile_type: TileType::Unknown,
      entropy: usize::MAX,
      possible_states: vec![],
      index: -1,
    }
  }

  pub fn initialise(&mut self, terrain_type: TerrainType, tile_type: TileType, states: &Vec<TerrainState>) {
    if self.is_initialised {
      panic!("Attempting to initialise a cell that already has been initialised");
    }
    // Uncomment the below to monitor specific cells
    // let points = vec![Point::new_internal_grid(9, 6)];
    // if points.contains(&self.ig) {
    //   self.is_being_monitored = true;
    // }
    if self.is_being_monitored {
      debug!(
        "Initialising ig{:?} as a [{:?}] cell with {:?} possible state(s): {:?}",
        self.ig,
        terrain_type,
        states.len(),
        states.iter().map(|s| s.name).collect::<Vec<ObjectName>>()
      );
    }
    self.is_initialised = true;
    self.terrain = terrain_type;
    self.tile_type = tile_type;
    self.possible_states = states.clone();
    self.entropy = self.possible_states.len();
  }

  pub fn clone_and_reduce(
    &self,
    reference_cell: &Cell,
    where_is_reference: &Connection,
  ) -> Result<(bool, Self), PropagationFailure> {
    let where_is_self_for_reference = where_is_reference.opposite();
    let permitted_state_names = get_permitted_new_states(&reference_cell, &where_is_self_for_reference);

    let mut updated_possible_states = Vec::new();
    for possible_state_self in &self.possible_states {
      if permitted_state_names.contains(&possible_state_self.name) {
        updated_possible_states.push(possible_state_self.clone());
      };
    }

    let mut clone = self.clone();
    clone.possible_states = updated_possible_states;
    clone.entropy = self.possible_states.len();
    log_result(
      true,
      reference_cell,
      where_is_reference,
      where_is_self_for_reference,
      self,
      &mut clone,
      &permitted_state_names,
    );

    match clone.possible_states.len() {
      0 => Err(PropagationFailure {}),
      _ => Ok((self.possible_states.len() != clone.possible_states.len(), clone)),
    }
  }

  pub fn collapse(&mut self, rng: &mut StdRng) {
    let possible_states_count = self.possible_states.len();
    let state = if possible_states_count == 1 {
      &self.possible_states[0]
    } else {
      let total_weight: i32 = self.possible_states.iter().map(|state| state.weight).sum();
      let mut target = rng.gen_range(0..total_weight);
      let mut selected_state = None;
      let mut states_logs = vec![];
      let initial_target = target;

      for state in &self.possible_states {
        if target < state.weight {
          selected_state = Some(state);
          break;
        }
        states_logs.push(format!("│  • State [{:?}] has a weight of {}", state.name, state.weight));
        target -= state.weight;
      }
      let selected_state = selected_state.expect("Failed to get selected state");

      log_collapse_result(
        &self,
        possible_states_count,
        total_weight,
        &mut states_logs,
        selected_state,
        initial_target,
      );

      selected_state
    };

    if self.is_being_monitored {
      debug!(
        "Collapsed ig{:?} to [{:?}] with previous entropy {} and {} states: {:?}",
        self.ig,
        state.name,
        self.entropy,
        self.possible_states.len(),
        self.possible_states.iter().map(|s| s.name).collect::<Vec<ObjectName>>()
      );
    }

    self.index = state.index;
    self.is_collapsed = true;
    self.entropy = 0;
    self.possible_states = vec![state.clone()];
  }

  pub fn verify(&self, reference_cell: &Cell, where_is_reference: &Connection) -> Result<(), PropagationFailure> {
    let where_is_self_for_reference = where_is_reference.opposite();
    let permitted_state_names = get_permitted_new_states(&reference_cell, &where_is_self_for_reference);

    if !permitted_state_names.contains(&self.possible_states[0].name) {
      log_result(
        false,
        reference_cell,
        where_is_reference,
        where_is_self_for_reference,
        self,
        &mut self.clone(),
        &permitted_state_names,
      );
      Err(PropagationFailure {})
    } else {
      Ok(())
    }
  }

  pub fn is_border_cell(&self, grid_size: usize) -> bool {
    let x = self.ig.x;
    let y = self.ig.y;
    let grid_size = grid_size as i32;
    x == 0 || y == 0 || x == grid_size - 1 || y == grid_size - 1
  }
}

fn get_permitted_new_states(reference_cell: &Cell, where_is_self_for_reference: &Connection) -> Vec<ObjectName> {
  reference_cell
    .possible_states
    .iter()
    .flat_map(|possible_state_reference| {
      possible_state_reference
        .permitted_neighbours
        .iter()
        .filter(|(connection, _)| connection == where_is_self_for_reference)
        .flat_map(|(_, names)| names.iter().cloned())
    })
    .collect()
}

fn log_result(
  is_update: bool,
  reference_cell: &Cell,
  where_is_reference: &Connection,
  where_is_self_for_reference: Connection,
  old_cell: &Cell,
  new_cell: &mut Cell,
  new_permitted_states: &Vec<ObjectName>,
) {
  if !new_cell.is_being_monitored && !reference_cell.is_being_monitored {
    return;
  }

  let old_possible_states_count = old_cell.possible_states.len();
  let new_possible_states_count = new_cell.possible_states.len();
  let new_possible_states_names = new_cell.possible_states.iter().map(|s| s.name).collect::<Vec<ObjectName>>();

  if old_possible_states_count != new_possible_states_count
    && is_update
    && new_cell.is_being_monitored
    && new_possible_states_count < 3
  {
    debug!(
      "Reduced possible states of ig{:?} from {} to {}: {:?}",
      new_cell.ig,
      old_possible_states_count,
      new_cell.possible_states.len(),
      new_possible_states_names
    );
  }

  if new_cell.possible_states.is_empty() {
    error!(
      "Failed to find any possible states for ig{:?} ({:?}, at [{:?}] of latter) during {} with ig{:?} ({:?})",
      new_cell.ig,
      old_cell.terrain,
      where_is_reference,
      if is_update { "update" } else { "verification" },
      reference_cell.ig,
      reference_cell.terrain,
    );
  }

  if new_possible_states_count <= 1 {
    debug!(
      "┌─|| Summary of the [{}] process for ig{:?}",
      if is_update { "update" } else { "verification" },
      old_cell.ig
    );
    debug!(
      "| - THIS cell is at ig{:?} which is at the [{:?}] of the reference cell",
      old_cell.ig, where_is_reference
    );
    debug!(
      "| - THIS cell had {:?} possible state(s): {:?}",
      old_possible_states_count,
      old_cell.possible_states.iter().map(|s| s.name).collect::<Vec<ObjectName>>()
    );
    debug!(
      "| - The REFERENCE cell is at ig{:?} which is at the [{:?}] of this cell)",
      reference_cell.ig, where_is_self_for_reference
    );
    debug!(
      "| - The REFERENCE cell has the following {} possible state(s): {:?}",
      reference_cell.possible_states.len(),
      reference_cell
        .possible_states
        .iter()
        .map(|s| s.name)
        .collect::<Vec<ObjectName>>()
    );
    if reference_cell.possible_states.len() == 1 {
      if let Some((_, neighbours)) = reference_cell.possible_states[0]
        .permitted_neighbours
        .iter()
        .find(|(connection, _)| *connection == where_is_self_for_reference)
      {
        debug!(
          "| - The relevant rule for a [{:?}] neighbour of the REFERENCE cell is: {:?}",
          where_is_reference, neighbours
        );
      } else {
        warn!("| - The relevant rule for only possible state of the REFERENCE cell does not exist");
      }
    }
    debug!(
      "| - The permitted new states were determined to be: {:?}",
      new_permitted_states
    );
    debug!(
      "└─> Result: THIS cell has {} new possible state(s): {:?}",
      new_cell.possible_states.len(),
      new_possible_states_names
    );
    debug!("")
  }
}

fn log_collapse_result(
  cell: &Cell,
  possible_states_count: usize,
  total_weight: i32,
  states_logs: &mut Vec<String>,
  selected_state: &TerrainState,
  target: i32,
) {
  if cell.is_being_monitored {
    debug!(
      "┌─|| There are {} possible states for [{:?}] terrain cell of type [{:?}] at ig{:?}",
      possible_states_count, cell.terrain, cell.tile_type, cell.ig
    );
    debug!("├─ The randomly selected target is {} out of {}", target, total_weight);
    debug!(
      "├─ Skipped the following {} states while iterating towards the target:",
      states_logs.len()
    );
    for log in states_logs {
      debug!("{}", log);
    }
    debug!(
      "└─> Selected state for ig{:?} is [{:?}] with a weight of {}",
      cell.ig, selected_state.name, selected_state.weight
    );
  }
}
