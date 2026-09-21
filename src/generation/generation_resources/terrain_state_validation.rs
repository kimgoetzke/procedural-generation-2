use crate::generation::model::{TerrainType, TileType};
use crate::generation::object::model::{Connection, ObjectName, TerrainState};
use bevy::log::{debug, error};
use bevy::platform::collections::{HashMap, HashSet};

/// Validates the terrain state map in a basic way. This function checks for the following:
/// - The map must not contain [`TileType::Unknown`] for any [`TerrainType`]
/// - Each state must not have asymmetric neighbour rules (i.e. a state that allows a neighbour in one direction
///   but the neighbour state does not allow the original state in the opposite direction) - however, paths and
///   [`ObjectName::Empty`] are ignored
/// - Each state must not have duplicate neighbours in the same direction
/// - Each state must not have duplicate [`Connection`]s (i.e. same direction defined multiple times)
/// - Each state must not have missing [`Connection`] definitions
pub(in crate::generation::generation_resources) fn validate_terrain_state_map(
  terrain_state_map: &HashMap<TerrainType, HashMap<TileType, Vec<TerrainState>>>,
) {
  let mut errors = HashSet::new();
  let state_lookup_map = build_state_lookup_map(terrain_state_map);
  for (terrain, state_map) in terrain_state_map {
    for (tile_type, states) in state_map {
      if let Err(error_msg) = validate_tile_type(tile_type, terrain) {
        errors.insert(error_msg);
        continue;
      }
      for state in states {
        validate_terrain_state(state, *terrain, &state_lookup_map, terrain_state_map, &mut errors);
      }
    }
  }

  if !errors.is_empty() {
    error!("Found [{}] validation errors in terrain state map:", errors.len());
    for (i, error) in errors.iter().enumerate() {
      error!("- {}. {}", i + 1, error);
    }
    panic!("Terrain state map failed validation - please fix the errors above before proceeding");
  } else if errors.is_empty() {
    debug!("✅  Terrain state map passed validation");
  }
}

/// Builds a lookup map for terrain states. This is used to quickly look up terrain states by their terrain type,
/// tile type, and object name.
fn build_state_lookup_map(
  terrain_state_map: &HashMap<TerrainType, HashMap<TileType, Vec<TerrainState>>>,
) -> HashMap<(TerrainType, TileType, ObjectName), &TerrainState> {
  terrain_state_map
    .iter()
    .flat_map(|(terrain, state_map)| {
      state_map
        .iter()
        .flat_map(move |(tile_type, states)| states.iter().map(move |state| ((*terrain, *tile_type, state.name), state)))
    })
    .collect()
}

/// Validates the tile type. Returns an error if the tile type is [`TileType::Unknown`] since this tile type is only
/// used to signal an error in the generation logic and should not be present in the terrain state map.
fn validate_tile_type(tile_type: &TileType, terrain: &TerrainType) -> Result<(), String> {
  match tile_type {
    TileType::Unknown => Err(format!(
      "Found tile type [Unknown] for terrain [{:?}], which is not allowed",
      terrain
    )),
    _ => Ok(()),
  }
}

/// Validates the neighbours of a given terrain state. See documentation for functions called within this function
/// for more details.
fn validate_terrain_state(
  state: &TerrainState,
  terrain: TerrainType,
  state_lookup: &HashMap<(TerrainType, TileType, ObjectName), &TerrainState>,
  terrain_state_map: &HashMap<TerrainType, HashMap<TileType, Vec<TerrainState>>>,
  errors: &mut HashSet<String>,
) {
  check_for_asymmetric_rules(state, terrain, state_lookup, terrain_state_map, errors);
  check_for_duplicate_neighbours(state, terrain, errors);
  check_for_duplicate_connections(state, terrain, errors);
  check_for_missing_connections(state, terrain, errors);
}

/// Adds an error to `errors` for each asymmetric neighbour rules in the given terrain state. Asymmetry refers to
/// a state allowing a neighbour in one direction, but the neighbour state not allowing the original state in the
/// opposite direction.
///
/// This is only checked for non-path/-building objects, as paths/buildings are allowed to have asymmetric connections
/// because they are calculated and "collapsed" before the wave function collapse algorithm even runs. As a result,
/// only non-path/-building objects need to know that they are allowed to be placed next to a path or building object
/// and no rules for the opposite are required since they will never be evaluated.
fn check_for_asymmetric_rules(
  state: &TerrainState,
  terrain: TerrainType,
  state_lookup_map: &HashMap<(TerrainType, TileType, ObjectName), &TerrainState>,
  terrain_state_map: &HashMap<TerrainType, HashMap<TileType, Vec<TerrainState>>>,
  errors: &mut HashSet<String>,
) {
  let all_terrain_tile_combinations: Vec<(TerrainType, TileType)> = terrain_state_map
    .iter()
    .flat_map(|(terrain, state_map)| state_map.keys().map(move |tile_type| (*terrain, *tile_type)))
    .collect();

  for (connection, permitted_neighbours) in &state.permitted_neighbours {
    let opposite_connection = connection.opposite();
    for &neighbour_object_name in permitted_neighbours {
      let has_reciprocal = all_terrain_tile_combinations
        .iter()
        .filter_map(|(terrain_type, tile_type)| state_lookup_map.get(&(*terrain_type, *tile_type, neighbour_object_name)))
        .any(|neighbour_state| {
          neighbour_state
            .permitted_neighbours
            .iter()
            .any(|(c, neighbours)| *c == opposite_connection && neighbours.contains(&state.name))
        });
      if !has_reciprocal && !neighbour_object_name.is_path() && !neighbour_object_name.is_settlement_structure() {
        errors.insert(format!(
          "Asymmetric [{:?}] neighbour rule: [{:?}] allows [{:?}] on its [{:?}], but [{:?}] doesn't allow [{:?}] on its [{:?}]",
          terrain,
          state.name,
          neighbour_object_name,
          connection,
          neighbour_object_name,
          state.name,
          opposite_connection
        ));
      }
    }
  }
}

/// Adds an error to `errors` if there are duplicate neighbours - i.e. [`ObjectName`]s - in
/// [`TerrainState::permitted_neighbours`].
fn check_for_duplicate_neighbours(state: &TerrainState, terrain: TerrainType, errors: &mut HashSet<String>) {
  for (connection, permitted_neighbours) in &state.permitted_neighbours {
    let unique_neighbours: HashSet<&ObjectName> = permitted_neighbours.iter().collect();
    if unique_neighbours.len() != permitted_neighbours.len() {
      errors.insert(format!(
        "Duplicate neighbours found in [{:?}] for [{:?}] [{:?}]",
        terrain, state.name, connection,
      ));
    }
  }
}

/// Adds an error to `errors` if there are duplicate [`Connection`]s in [`TerrainState::permitted_neighbours`].
fn check_for_duplicate_connections(state: &TerrainState, terrain: TerrainType, errors: &mut HashSet<String>) {
  let connections: Vec<Connection> = state.permitted_neighbours.iter().map(|(c, _)| *c).collect();
  let unique_connections: HashSet<_> = connections.iter().collect();

  if unique_connections.len() != connections.len() {
    errors.insert(format!(
      "Duplicate connection directions found for [{:?}] [{:?}]: {:?}",
      terrain, state.name, connections
    ));
  }
}

/// Adds an error to `errors` if not all four cardinal directions are defined as [`Connection`]s in
/// [`TerrainState::permitted_neighbours`].
fn check_for_missing_connections(state: &TerrainState, terrain: TerrainType, errors: &mut HashSet<String>) {
  const ALL_CONNECTIONS: [Connection; 4] = [Connection::Top, Connection::Right, Connection::Bottom, Connection::Left];
  let defined_connections: HashSet<Connection> = state.permitted_neighbours.iter().map(|(c, _)| *c).collect();
  for connection in &ALL_CONNECTIONS {
    if !defined_connections.contains(connection) {
      errors.insert(format!(
        "Connection definition for [{:?}] is missing for [{:?}] [{:?}]",
        connection, terrain, state.name
      ));
    }
  }
}
