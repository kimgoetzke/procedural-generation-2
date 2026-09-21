use crate::generation::model::{Climate, TerrainType, TileType};
use crate::generation::object::model::{ObjectName, TerrainState};
use bevy::asset::{Asset, Assets, Handle};
use bevy::log::{debug, trace};
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::{Reflect, Res, ResMut, Resource, TypePath};
use std::fmt;
use std::fmt::{Display, Formatter};
use strum::IntoEnumIterator;

#[derive(Resource, Default, Debug, Clone)]
pub(in crate::generation::generation_resources) struct TerrainRuleSetHandle(pub Vec<Handle<TerrainRuleSet>>);

#[derive(serde::Deserialize, Asset, TypePath, Debug, Clone)]
pub(in crate::generation::generation_resources) struct TerrainRuleSet {
  terrain: TerrainType,
  states: Vec<TerrainState>,
}

impl Default for TerrainRuleSet {
  fn default() -> Self {
    Self {
      terrain: TerrainType::Any,
      states: vec![],
    }
  }
}

impl Display for TerrainRuleSet {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "[{:?}] terrain rule set with {} states", self.terrain, self.states.len())
  }
}

#[derive(Resource, Default, Debug, Clone)]
pub(in crate::generation::generation_resources) struct TileTypeRuleSetHandle(pub Handle<TileTypeRuleSet>);

#[derive(serde::Deserialize, Asset, TypePath, Debug, Clone)]
pub(in crate::generation::generation_resources) struct TileTypeRuleSet {
  states: Vec<TileTypeState>,
}

impl Display for TileTypeRuleSet {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "Tile type rule set with {} states", self.states.len())
  }
}

#[derive(serde::Deserialize, Debug, Clone, Reflect)]
struct TileTypeState {
  pub tile_type: TileType,
  pub permitted_self: Vec<ObjectName>,
}

#[derive(Resource, Default, Debug, Clone)]
pub(in crate::generation::generation_resources) struct ExclusionsRuleSetHandle(pub Handle<ExclusionsRuleSet>);

#[derive(serde::Deserialize, Asset, TypePath, Debug, Clone)]
pub(in crate::generation::generation_resources) struct ExclusionsRuleSet {
  states: Vec<ExclusionsState>,
}

impl Display for ExclusionsRuleSet {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "Exclusions rule set with {} states", self.states.len())
  }
}

#[derive(serde::Deserialize, Debug, Clone, Reflect)]
struct ExclusionsState {
  pub terrain: TerrainType,
  pub climate: Climate,
  pub excluded_objects: Vec<ObjectName>,
}

pub(in crate::generation::generation_resources) fn terrain_rules(
  terrain_rule_set_handle: Res<TerrainRuleSetHandle>,
  terrain_rule_set_assets: &mut ResMut<Assets<TerrainRuleSet>>,
) -> HashMap<TerrainType, Vec<TerrainState>> {
  let mut rule_sets = HashMap::new();
  for handle in terrain_rule_set_handle.0.iter() {
    if let Some(rule_set) = terrain_rule_set_assets.remove(handle) {
      debug!("Loaded: {}", rule_set);
      rule_sets.insert(rule_set.terrain, rule_set.states);
    }
  }
  if let Some(any_rule_set) = rule_sets.remove(&TerrainType::Any) {
    debug!(
      "Found [Any] terrain rule set with [{}] state(s) and will extend each of the other rule sets accordingly",
      any_rule_set.len()
    );
    for (terrain, states) in rule_sets.iter_mut() {
      states.splice(0..0, any_rule_set.iter().cloned());
      debug!(
        "Extended [{}] rule set by [{}], it now has [{}] states",
        terrain,
        any_rule_set.len(),
        states.len()
      );
    }
    rule_sets.insert(TerrainType::Any, any_rule_set);
  }

  rule_sets
}

pub(in crate::generation::generation_resources) fn tile_type_rules(
  tile_type_rule_set_handle: Res<TileTypeRuleSetHandle>,
  tile_type_rule_set_assets: &mut ResMut<Assets<TileTypeRuleSet>>,
) -> HashMap<TileType, Vec<ObjectName>> {
  if let Some(rule_set) = tile_type_rule_set_assets.remove(&tile_type_rule_set_handle.0) {
    debug!("Loaded: Tile type rule set for [{}] tiles", rule_set.states.len());
    let mut rule_sets = HashMap::new();
    for state in rule_set.states {
      rule_sets.insert(state.tile_type, state.permitted_self);
    }
    return rule_sets;
  }

  HashMap::new()
}

pub(in crate::generation::generation_resources) fn exclusion_rules(
  exclusion_rule_set_handle: Res<ExclusionsRuleSetHandle>,
  exclusion_rule_set_assets: &mut ResMut<Assets<ExclusionsRuleSet>>,
) -> HashMap<(TerrainType, Climate), Vec<ObjectName>> {
  if let Some(rule_set) = exclusion_rule_set_assets.remove(&exclusion_rule_set_handle.0) {
    debug!(
      "Loaded: Exclusions rule set for [{}] terrain-climate combinations",
      rule_set.states.len()
    );
    let mut rule_sets = HashMap::new();
    for state in rule_set.states {
      rule_sets.insert((state.terrain, state.climate), state.excluded_objects);
    }
    return rule_sets;
  }

  HashMap::new()
}

/// Resolves the terrain rules and tile type rules into a single map that associates terrain types with tile types and
/// their possible states.
///
/// Note: [`TileType::Unknown`] is filtered out, as it is not a valid tile type and is only used to signal
/// an error in the generation logic. This [`TileType`] will not cause panics but will be rendered as a bright,
/// single-coloured tile to indicate the error.
pub(in crate::generation::generation_resources) fn resolve_rules_to_terrain_states_map(
  terrain_rules: HashMap<TerrainType, Vec<TerrainState>>,
  tile_type_rules: HashMap<TileType, Vec<ObjectName>>,
) -> HashMap<TerrainType, HashMap<TileType, Vec<TerrainState>>> {
  let mut terrain_state_map: HashMap<TerrainType, HashMap<TileType, Vec<TerrainState>>> = HashMap::new();
  for terrain in TerrainType::iter() {
    let relevant_terrain_rules = terrain_rules
      .get(&terrain)
      .unwrap_or_else(|| panic!("Failed to find rule set for [{:?}] terrain", terrain));
    let resolved_rules_for_terrain: HashMap<TileType, Vec<TerrainState>> = TileType::iter()
      .filter(|&t| t != TileType::Unknown)
      .map(|tile_type| {
        let all_rules_for_tile_type = tile_type_rules
          .get(&tile_type)
          .unwrap_or_else(|| panic!("Failed to find rule set for [{:?}] tile type", tile_type));
        let resolved_rules_for_tile_type = relevant_terrain_rules
          .iter()
          .filter(|rule| all_rules_for_tile_type.contains(&rule.name))
          .cloned()
          .collect();

        (tile_type, resolved_rules_for_tile_type)
      })
      .collect();
    trace!(
      "Resolved [{}] rules for [{:?}] terrain type: {:?}",
      resolved_rules_for_terrain.values().map(|ts| ts.len()).sum::<usize>(),
      terrain,
      resolved_rules_for_terrain
        .iter()
        .map(|(k, v)| (k, v.len()))
        .collect::<HashMap<&TileType, usize>>()
    );
    terrain_state_map.insert(terrain, resolved_rules_for_terrain);
  }
  debug!(
    "Resolved [{}] rules for [{}] terrain types",
    terrain_state_map
      .values()
      .map(|tile_map| tile_map.values().map(|v| v.len()).sum::<usize>())
      .sum::<usize>(),
    terrain_state_map.len()
  );

  terrain_state_map
}

/// Turns a terrain state map into a terrain-climate state map by applying the exclusion rules. For each terrain type
/// and climate combination, the relevant exclusion rules are applied to filter out any excluded object names from
/// the terrain states.
pub(in crate::generation::generation_resources) fn apply_exclusions(
  exclusion_rules: HashMap<(TerrainType, Climate), Vec<ObjectName>>,
  terrain_state_map: HashMap<TerrainType, HashMap<TileType, Vec<TerrainState>>>,
) -> HashMap<(TerrainType, Climate), HashMap<TileType, Vec<TerrainState>>> {
  let mut terrain_climate_state_map = HashMap::new();
  for terrain in terrain_state_map.keys() {
    for climate in Climate::iter() {
      let excluded_objects = exclusion_rules.get(&(*terrain, climate)).cloned().unwrap_or_default();
      let mut object_count_before = 0;
      let mut cloned_states = terrain_state_map.get(terrain).expect("Terrain must exist").clone();
      if !excluded_objects.is_empty() {
        debug!(
          "Applying up to [{}] exclusions rules to [{:?}] terrain in [{:?}] climate",
          excluded_objects.len(),
          terrain,
          climate
        );
        let distinct_objects = cloned_states
          .values()
          .flat_map(|states| states.iter().map(|s| &s.name))
          .collect::<HashSet<&ObjectName>>();
        object_count_before = distinct_objects.len();
        trace!(" ├─> [{}] objects before: {:?}", object_count_before, distinct_objects);
      }
      cloned_states
        .iter_mut()
        .for_each(|entry| entry.1.retain(|state| !excluded_objects.contains(&state.name)));
      terrain_climate_state_map.insert((*terrain, climate), cloned_states.clone());
      if !excluded_objects.is_empty() {
        trace!(" ├─> [{}] exclusions to be applied", excluded_objects.len());
        let distinct_objects_after = cloned_states
          .values()
          .flat_map(|states| states.iter().map(|s| &s.name))
          .collect::<HashSet<&ObjectName>>();
        trace!(
          " ├─> [{}] objects after: {:?}",
          distinct_objects_after.len(),
          distinct_objects_after
        );
        trace!(
          " └─> [{}] objects removed",
          object_count_before - distinct_objects_after.len()
        );
        trace!("");
      }
    }
  }

  terrain_climate_state_map
}
