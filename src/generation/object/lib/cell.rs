use crate::constants::CHUNK_SIZE;
use crate::coords::Point;
use crate::coords::point::InternalGrid;
use crate::generation::lib::{TerrainType, TileType};
use crate::generation::object::lib::terrain_state::TerrainState;
use crate::generation::object::lib::tile_below::TileBelow;
use crate::generation::object::lib::{Connection, ObjectName};
use bevy::log::*;
use bevy::prelude::Reflect;
use rand::Rng;
use rand::prelude::StdRng;
use std::sync::{Arc, Mutex};

pub struct PropagationFailure {}

pub type CellRef = Arc<Mutex<Cell>>;

/// A [`Cell`] is a "placeholder" for an object. It is used in the [`ObjectGrid`][og]. This struct is used to represent
/// a cell in the grid that can be collapsed to a single state. Once all [`Cell`]s in an [`ObjectGrid`][og] have been
/// collapsed, they will be converted to [`ObjectData`][od]s which are then used to spawn object sprites in the world.
/// A [`Cell`] is indirectly linked to an underlying [`Tile`][t] through its [`TerrainType`] and  [`TileType`] fields.
/// Each [`Tile`][t] on a flat terrain plane has exactly 0 or 1 [`Cell`]s associated with it.
///
/// [og]: crate::generation::object::lib::ObjectGrid
/// [od]: crate::generation::object::lib::ObjectData
/// [t]: crate::generation::lib::Tile
#[derive(Debug, Clone, Reflect)]
pub struct Cell {
  // General fields
  pub ig: Point<InternalGrid>,
  index: i32,
  terrain: TerrainType,
  tile_type: TileType,
  #[reflect(ignore)]
  pub tile_below: Option<TileBelow>,
  // Pathfinding specific fields
  #[reflect(ignore)]
  neighbours: Vec<CellRef>,
  #[reflect(ignore)]
  connection: Box<Option<CellRef>>,
  g: f32,
  h: f32,
  is_walkable_connection: bool,
  is_walkable_tile: bool,
  // Wave function collapse specific fields
  is_collapsed: bool,
  is_initialised: bool,
  is_being_monitored: bool,
  entropy: usize,
  possible_states: Vec<TerrainState>,
}

impl PartialEq for Cell {
  fn eq(&self, other: &Self) -> bool {
    self.ig == other.ig
  }
}

impl Cell {
  pub fn new(x: i32, y: i32) -> Self {
    Cell {
      ig: Point::new_internal_grid(x, y),
      index: -1,
      terrain: TerrainType::Any,
      tile_type: TileType::Unknown,
      tile_below: None,
      neighbours: vec![],
      connection: Box::new(None),
      g: 0.0,
      h: 0.0,
      is_walkable_connection: true,
      is_walkable_tile: true,
      is_collapsed: false,
      is_initialised: false,
      is_being_monitored: false,
      entropy: usize::MAX,
      possible_states: vec![],
    }
  }

  pub fn initialise(
    &mut self,
    terrain_type: TerrainType,
    tile_type: TileType,
    states: &Vec<TerrainState>,
    lower_layer_info: Vec<(TerrainType, TileType)>,
  ) {
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
        "Initialising {:?} as a [{:?}] cell with {:?} possible state(s): {:?}",
        self.ig,
        terrain_type,
        states.len(),
        states.iter().map(|s| s.name).collect::<Vec<ObjectName>>()
      );
    }
    self.is_initialised = true;
    self.terrain = terrain_type;
    self.tile_type = tile_type;
    self.tile_below = if lower_layer_info.is_empty() {
      None
    } else {
      Some(TileBelow::new(lower_layer_info))
    };
    self.possible_states = states.clone();
    self.entropy = self.possible_states.len();
  }

  pub fn get_ig(&self) -> &Point<InternalGrid> {
    &self.ig
  }

  pub fn get_index(&self) -> i32 {
    self.index
  }

  pub fn add_neighbours(&mut self, neighbours: Vec<CellRef>) {
    for neighbour in neighbours {
      self.add_neighbour(neighbour);
    }
  }

  pub fn add_neighbour(&mut self, neighbour: CellRef) {
    let neighbour_ig = neighbour.try_lock().expect("Failed to lock cell to find").ig;
    if !self
      .neighbours
      .iter()
      .any(|n| n.try_lock().expect("Failed to lock neighbour").ig == neighbour_ig)
    {
      self.neighbours.push(neighbour);
    }
  }

  /// Returns the [`CellRef`]s of all neighbours of this cell if they are walkable. See [`Cell::calculate_is_walkable`]
  /// for the definition of walkable.
  pub fn get_walkable_neighbours(&self) -> Vec<CellRef> {
    self
      .neighbours
      .iter()
      .filter(|n| n.try_lock().expect("Failed to lock neighbour").is_walkable())
      .cloned()
      .collect::<Vec<CellRef>>()
  }

  /// Returns the [`CellRef`] that this cell is connected to, if any. Used to reconstruct the path from the start cell
  /// to the target cell after the pathfinding algorithm has completed.
  pub fn get_connection(&self) -> &Option<CellRef> {
    &self.connection
  }

  /// Sets the connection to another [`CellRef`], which is used to reconstruct the path from the start cell to the
  /// target.
  pub fn set_connection(&mut self, connection: &CellRef) {
    *self.connection = Some(connection.clone());
  }

  /// The distance from the start cell to this cell.
  pub fn get_g(&self) -> f32 {
    self.g
  }

  /// Sets the `G` cost which represents the distance from the start cell to this cell.
  pub fn set_g(&mut self, g: f32) {
    self.g = g;
  }

  /// The heuristic value, which is the estimated ("ideal") distance to reach the target cell from this cell. This
  /// value is always equal to or less than the actual distance to the target cell.
  pub fn get_h(&self) -> f32 {
    self.h
  }

  /// Sets the `H` cost i.e. heuristic value, which is the estimated distance to reach the target cell from this cell.
  pub fn set_h(&mut self, h: f32) {
    self.h = h;
  }

  /// The total cost of this cell, which is the sum of the distance from the start cell (`G`) and the heuristic
  /// value (`H`).
  pub fn get_f(&self) -> f32 {
    self.g + self.h
  }

  /// Returns whether this cell is walkable.
  pub fn is_walkable(&self) -> bool {
    self.is_walkable_tile
  }

  /// Calculates whether this cell is walkable based on its terrain type and tile type.
  pub fn calculate_is_walkable(&mut self) {
    self.is_walkable_connection = self.is_walkable_connection();
    self.is_walkable_tile = self.is_walkable_tile();
  }

  /// Same as [`Cell::is_walkable_tile`] but more lenient because there are many edge cases where a connection point in
  /// one chunk is clearly walkable, but in another chunk may not be (specifically when the tile type is of a partial
  /// fill type).
  fn is_walkable_connection(&self) -> bool {
    (self.terrain == TerrainType::Land1 && is_filled_or_filled_at_facing_edge(self.ig, self.tile_type)
      || self.terrain.gt(&TerrainType::Land1))
      && self.terrain != TerrainType::Any
  }

  fn is_walkable_tile(&self) -> bool {
    (self.terrain == TerrainType::Land1 && self.tile_type == TileType::Fill || self.terrain.gt(&TerrainType::Land1))
      && self.terrain != TerrainType::Any
  }

  pub fn is_valid_connection_point(&self) -> bool {
    if !self.is_walkable_connection {
      return false;
    }
    if is_filled_or_filled_at_facing_edge(self.ig, self.tile_type) {
      return true;
    }
    if self.terrain > TerrainType::Land1 && is_filled_at_facing_edge_incl_corner_types(self.ig, self.tile_type) {
      return true;
    }
    self.tile_below.as_ref().map_or(false, |tile_below| {
      let mut current = Some(tile_below);
      while let Some(below) = current {
        if is_filled_or_filled_at_facing_edge(self.ig, below.tile_type) && below.terrain >= TerrainType::Land1 {
          return true;
        }
        current = below.below.as_deref();
      }
      false
    })
  }

  pub fn log_tiles_below(&self) {
    if let Some(tile_below) = &self.tile_below {
      tile_below.log();
    } else {
      debug!("- This cell does not have a tile below it");
    }
  }

  pub fn is_collapsed(&self) -> bool {
    self.is_collapsed
  }

  pub fn get_entropy(&self) -> usize {
    self.entropy
  }

  pub fn get_possible_states(&self) -> &Vec<TerrainState> {
    &self.possible_states
  }

  /// Sets the possible states of this cell. Does NOT update the entropy and can therefore cause an *inconsistent*
  /// state. Only use this method if you know what you are doing. States should only be updated using
  /// [`Cell::clone_and_reduce`] or [`Cell::collapse`] as part of running the wave function collapse algorithm.
  pub fn override_possible_states(&mut self, states: Vec<TerrainState>) {
    self.possible_states = states;
  }

  pub fn clear_references(&mut self) {
    self.neighbours.clear();
    self.connection = Box::new(None);
    self.g = 0.0;
    self.h = 0.0;
  }

  /// Used outside the wave function collapse algorithm to set the cell as collapsed with a single state.
  pub fn set_collapsed(&mut self, object_name: ObjectName) {
    let i = object_name.get_index_for_path();
    self.index = i;
    self.is_collapsed = true;
    self.entropy = 0usize;
    self.possible_states = vec![TerrainState::new_with_no_neighbours(object_name, i, 1)];
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
      let mut target = rng.random_range(0..total_weight);
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
        "Collapsed {:?} to [{:?}] with previous entropy {} and {} states: {:?}",
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

fn is_filled_or_filled_at_facing_edge(ig: Point<InternalGrid>, tile_type: TileType) -> bool {
  (ig.x == 0
    && matches!(
      tile_type,
      TileType::LeftFill | TileType::OuterCornerTopRight | TileType::OuterCornerBottomRight
    ))
    || (ig.x == CHUNK_SIZE - 1
      && matches!(
        tile_type,
        TileType::RightFill | TileType::OuterCornerBottomLeft | TileType::OuterCornerTopLeft
      ))
    || (ig.y == 0
      && matches!(
        tile_type,
        TileType::TopFill | TileType::OuterCornerBottomLeft | TileType::OuterCornerBottomRight
      ))
    || (ig.y == CHUNK_SIZE - 1
      && matches!(
        tile_type,
        TileType::BottomFill | TileType::OuterCornerTopRight | TileType::OuterCornerTopLeft
      ))
    || tile_type == TileType::Fill
}

fn is_filled_at_facing_edge_incl_corner_types(ig: Point<InternalGrid>, tile_type: TileType) -> bool {
  use TileType::*;

  match tile_type {
    Fill | BottomFill | TopFill if ig.x == 0 || ig.x == CHUNK_SIZE - 1 => true,
    Fill | LeftFill | RightFill if ig.y == 0 || ig.y == CHUNK_SIZE - 1 => true,
    OuterCornerBottomRight | OuterCornerBottomLeft if ig.y == 0 => true,
    OuterCornerTopRight | OuterCornerTopLeft if ig.y == CHUNK_SIZE - 1 => true,
    OuterCornerBottomRight | OuterCornerTopRight if ig.x == 0 => true,
    OuterCornerBottomLeft | OuterCornerTopLeft if ig.x == CHUNK_SIZE - 1 => true,
    _ => false,
  }
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
      "Reduced possible states of {:?} from {} to {}: {:?}",
      new_cell.ig,
      old_possible_states_count,
      new_cell.possible_states.len(),
      new_possible_states_names
    );
  }

  if new_cell.possible_states.is_empty() {
    error!(
      "Failed to find any possible states for {:?} ({:?}, at [{:?}] of latter) during {} with {:?} ({:?})",
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
      "┌─|| Summary of the [{}] process for {:?}",
      if is_update { "update" } else { "verification" },
      old_cell.ig
    );
    debug!(
      "| - THIS cell is at {:?} which is at the [{:?}] of the reference cell",
      old_cell.ig, where_is_reference
    );
    debug!(
      "| - THIS cell had {:?} possible state(s): {:?}",
      old_possible_states_count,
      old_cell.possible_states.iter().map(|s| s.name).collect::<Vec<ObjectName>>()
    );
    debug!(
      "| - The REFERENCE cell is at {:?} which is at the [{:?}] of this cell)",
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
      "┌─|| There are {} possible states for [{:?}] terrain cell of type [{:?}] at {:?}",
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
      "└─> Selected state for {:?} is [{:?}] with a weight of {}",
      cell.ig, selected_state.name, selected_state.weight
    );
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  impl Cell {
    pub fn get_neighbours(&self) -> &Vec<CellRef> {
      &self.neighbours
    }
  }

  #[test]
  fn new_sets_correct_ig() {
    let ig = Point::default();
    let cell = Cell::new(ig.x, ig.y);
    assert_eq!(cell.get_ig(), &ig);
  }

  #[test]
  fn add_neighbour_only_adds_unique_neighbours() {
    let cell = Arc::new(Mutex::new(Cell::new(0, 0)));
    let neighbour = Arc::new(Mutex::new(Cell::new(0, 0)));
    let mut cell_guard = cell.lock().unwrap();
    cell_guard.add_neighbour(neighbour.clone());
    cell_guard.add_neighbour(neighbour.clone());
    assert_eq!(cell_guard.get_neighbours().len(), 1);
  }

  #[test]
  fn add_neighbours_adds_multiple_unique_neighbours() {
    let cell = Arc::new(Mutex::new(Cell::new(0, 0)));
    let neighbour1 = Arc::new(Mutex::new(Cell::new(1, 1)));
    let neighbour2 = Arc::new(Mutex::new(Cell::new(2, 2)));
    let neighbours = vec![neighbour1.clone(), neighbour2.clone(), neighbour1.clone()];
    let mut cell_guard = cell.lock().unwrap();
    cell_guard.add_neighbours(neighbours);
    assert_eq!(cell_guard.get_neighbours().len(), 2);
  }

  #[test]
  fn add_neighbours_adds_multiple_unique_neighbours_old() {
    let cell = Arc::new(Mutex::new(Cell::new(0, 0)));
    let neighbour1 = Arc::new(Mutex::new(Cell::new(1, 1)));
    let neighbour2 = Arc::new(Mutex::new(Cell::new(2, 2)));
    let mut cell_guard = cell.lock().unwrap();
    cell_guard.add_neighbours(vec![neighbour1.clone(), neighbour2.clone(), neighbour1.clone()]);
    assert_eq!(cell_guard.get_neighbours().len(), 2);
  }

  #[test]
  fn set_connection_sets_and_then_updates_connection() {
    let cell_ref = Arc::new(Mutex::new(Cell::new(1, 1)));
    let connection1 = Arc::new(Mutex::new(Cell::new(2, 2)));
    let mut cell_guard = cell_ref.lock().unwrap();
    cell_guard.set_connection(&connection1);
    assert!(
      cell_guard
        .get_connection()
        .as_ref()
        .map(|c| Arc::ptr_eq(c, &connection1))
        .unwrap_or(false)
    );

    let connection2 = Arc::new(Mutex::new(Cell::new(4, 8)));
    cell_guard.set_connection(&connection2);
    assert!(
      cell_guard
        .get_connection()
        .as_ref()
        .map(|c| Arc::ptr_eq(c, &connection2))
        .unwrap_or(false)
    );
  }

  #[test]
  fn is_walkable_returns_true_for_land1_only_if_filled() {
    let mut cell = Cell::new(0, 0);
    cell.terrain = TerrainType::Land1;
    cell.tile_type = TileType::Fill;
    cell.calculate_is_walkable();
    assert!(cell.is_walkable());

    cell.tile_type = TileType::BottomFill;
    cell.calculate_is_walkable();
    assert!(!cell.is_walkable());

    cell.tile_type = TileType::OuterCornerBottomLeft;
    cell.calculate_is_walkable();
    assert!(!cell.is_walkable());
  }

  #[test]
  fn is_walkable_returns_true_for_anything_above_land1() {
    let mut cell = Cell::new(0, 0);
    cell.terrain = TerrainType::Land2;
    cell.tile_type = TileType::Fill;
    cell.calculate_is_walkable();
    assert!(cell.is_walkable());

    cell.tile_type = TileType::BottomFill;
    cell.calculate_is_walkable();
    assert!(cell.is_walkable());

    cell.tile_type = TileType::OuterCornerBottomLeft;
    cell.calculate_is_walkable();
    assert!(cell.is_walkable());
  }

  #[test]
  fn is_walkable_returns_false_for_any_water() {
    let mut cell = Cell::new(0, 0);
    cell.terrain = TerrainType::ShallowWater;
    cell.tile_type = TileType::Fill;
    cell.calculate_is_walkable();
    assert!(!cell.is_walkable());

    cell.terrain = TerrainType::DeepWater;
    cell.calculate_is_walkable();
    assert!(!cell.is_walkable());
  }

  #[test]
  fn is_valid_connection_point_returns_false_if_self_is_not_walkable() {
    let mut cell = Cell::new(0, 0);
    cell.is_walkable_connection = false;
    assert!(!cell.is_valid_connection_point());
  }

  #[test]
  fn is_valid_connection_point_returns_true_if_tile_type_is_fill() {
    let mut cell = Cell::new(0, 0);
    cell.is_walkable_connection = true;
    cell.tile_type = TileType::Fill;
    assert!(cell.is_valid_connection_point());
  }

  #[test]
  fn is_valid_connection_point_returns_true_if_any_below_tile_is_filled_and_land1_or_higher() {
    let mut cell = Cell::new(0, 0);
    cell.is_walkable_connection = true;
    cell.tile_type = TileType::Unknown;
    cell.tile_below = Some(TileBelow {
      terrain: TerrainType::Land2,
      tile_type: TileType::Unknown,
      below: Some(Box::new(TileBelow::from(TerrainType::Land1, TileType::Fill, None))),
    });
    assert!(cell.is_valid_connection_point());
  }

  #[test]
  fn is_valid_connection_point_returns_true_if_facing_edge_and_land1_or_higher() {
    let mut cell = Cell::new(0, 4);
    cell.is_walkable_connection = true;
    cell.tile_type = TileType::OuterCornerBottomRight;
    cell.tile_below = None;
    assert!(cell.is_valid_connection_point());
  }

  #[test]
  fn is_valid_connection_point_returns_true_if_below_is_facing_edge_and_land1_or_higher() {
    let mut cell = Cell::new(0, 4);
    cell.is_walkable_connection = true;
    cell.tile_type = TileType::Unknown;
    cell.tile_below = Some(TileBelow::from(TerrainType::Land1, TileType::LeftFill, None));
    assert!(cell.is_valid_connection_point());
  }

  #[test]
  fn is_valid_connection_point_returns_false_if_no_below_tile_is_filled() {
    let mut cell = Cell::new(0, 0);
    cell.is_walkable_connection = true;
    cell.tile_type = TileType::Unknown;
    cell.tile_below = Some(TileBelow {
      terrain: TerrainType::Land3,
      tile_type: TileType::Unknown,
      below: Some(Box::new(TileBelow::from(TerrainType::Land2, TileType::Unknown, None))),
    });
    assert!(!cell.is_valid_connection_point());
  }

  #[test]
  fn is_valid_connection_point_returns_false_if_only_filled_tile_below_is_water() {
    let mut cell = Cell::new(0, 0);
    cell.is_walkable_connection = true;
    cell.tile_type = TileType::Unknown;
    cell.tile_below = Some(TileBelow {
      terrain: TerrainType::Land1,
      tile_type: TileType::Unknown,
      below: Some(Box::new(TileBelow::from(TerrainType::ShallowWater, TileType::Fill, None))),
    });
    assert!(!cell.is_valid_connection_point());
  }

  #[test]
  fn is_valid_connection_point_returns_false_if_no_tile_below() {
    let mut cell = Cell::new(0, 0);
    cell.is_walkable_connection = true;
    cell.tile_type = TileType::Unknown;
    cell.tile_below = None;
    assert!(!cell.is_valid_connection_point());
  }
}
