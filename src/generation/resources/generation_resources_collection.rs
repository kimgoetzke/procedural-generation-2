use crate::constants::*;
use crate::generation::lib::{AssetCollection, AssetPack, GenerationResourcesCollection, TerrainType, TileType};
use crate::generation::object::lib::{Connection, ObjectName, TerrainState};
use crate::states::AppState;
use bevy::app::{App, Plugin, Startup, Update};
use bevy::asset::{Asset, AssetServer, Assets, Handle, LoadState};
use bevy::log::*;
use bevy::math::UVec2;
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::{
  Commands, IntoScheduleConfigs, NextState, OnExit, Reflect, Res, ResMut, Resource, TextureAtlasLayout, TypePath, in_state,
};
use bevy_common_assets::ron::RonAssetPlugin;
use std::fmt;
use std::fmt::{Display, Formatter};
use strum::IntoEnumIterator;

/// This plugin is responsible for loading and managing the resources - e.g. sprites and rule sets - required for the
/// generation process. The purpose of this plugin is to ensure that all necessary assets are loaded, preprocessed, and
/// initialised before the generation process starts.
///
/// At its core, this plugin adds the [`GenerationResourcesCollection`] resource, making it available to the rest of the
/// application.
///
/// In terms of process, it works as follows:
/// 1. The plugin loads the rule sets for terrain and tile types from the file system. At this point, the application is
///    in the [`AppState::Loading`] state. See [`load_rule_sets_system`].
/// 2. While in this state, it checks the loading state of these assets and waits until they are fully loaded, then
///    it transitions the state to [`AppState::Initialising`]. See [`check_loading_state_system`].
/// 3. Upon transitioning to the initialising state (i.e. [`OnExit`] of [`AppState::Loading`]), it finally
///    initialises the [`GenerationResourcesCollection`] resource. See [`initialise_resources_system`].
pub struct GenerationResourcesCollectionPlugin;

impl Plugin for GenerationResourcesCollectionPlugin {
  fn build(&self, app: &mut App) {
    app
      .init_resource::<GenerationResourcesCollection>()
      .add_plugins((
        RonAssetPlugin::<TerrainRuleSet>::new(&["terrain.ruleset.ron"]),
        RonAssetPlugin::<TileTypeRuleSet>::new(&["tile-type.ruleset.ron"]),
      ))
      .add_systems(Startup, load_rule_sets_system)
      .add_systems(Update, check_loading_state_system.run_if(in_state(AppState::Loading)))
      .add_systems(OnExit(AppState::Loading), initialise_resources_system);
  }
}

#[derive(Resource, Default, Debug, Clone)]
struct TerrainRuleSetHandle(Vec<Handle<TerrainRuleSet>>);

#[derive(serde::Deserialize, Asset, TypePath, Debug, Clone)]
struct TerrainRuleSet {
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
struct TileTypeRuleSetHandle(Handle<TileTypeRuleSet>);

#[derive(serde::Deserialize, Asset, TypePath, Debug, Clone)]
struct TileTypeRuleSet {
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

fn load_rule_sets_system(mut commands: Commands, asset_server: Res<AssetServer>) {
  let mut rule_set_handles = Vec::new();
  for terrain_type in TerrainType::iter() {
    let path = format!("objects/{}.terrain.ruleset.ron", terrain_type.to_string().to_lowercase());
    let handle = asset_server.load(path);
    rule_set_handles.push(handle);
  }
  let any_handle = asset_server.load("objects/any.terrain.ruleset.ron");
  rule_set_handles.push(any_handle);
  commands.insert_resource(TerrainRuleSetHandle(rule_set_handles));
  let handle = asset_server.load("objects/all.tile-type.ruleset.ron");
  commands.insert_resource(TileTypeRuleSetHandle(handle));
}

fn check_loading_state_system(
  asset_server: Res<AssetServer>,
  terrain_handles: Res<TerrainRuleSetHandle>,
  tile_type_handle: Res<TileTypeRuleSetHandle>,
  mut state: ResMut<NextState<AppState>>,
) {
  for handle in &terrain_handles.0 {
    if is_loading(asset_server.get_load_state(handle)) {
      info_once!("Waiting for assets to load...");
      return;
    }
  }
  if is_loading(asset_server.get_load_state(&tile_type_handle.0)) {
    info_once!("Waiting for assets to load...");
    return;
  }
  state.set(AppState::Initialising);
}

fn is_loading(loading_state: Option<LoadState>) -> bool {
  if let Some(state) = loading_state {
    return match state {
      LoadState::NotLoaded | LoadState::Loading => true,
      LoadState::Failed(e) => panic!("Failed to load assets: {:?}", e),
      _ => false,
    };
  };
  true
}

fn initialise_resources_system(
  asset_server: Res<AssetServer>,
  mut layouts: ResMut<Assets<TextureAtlasLayout>>,
  mut asset_collection: ResMut<GenerationResourcesCollection>,
  terrain_rule_set_handle: Res<TerrainRuleSetHandle>,
  mut terrain_rule_set_assets: ResMut<Assets<TerrainRuleSet>>,
  tile_type_rule_set_handle: Res<TileTypeRuleSetHandle>,
  mut tile_type_rule_set_assets: ResMut<Assets<TileTypeRuleSet>>,
) {
  // Placeholder tile set
  let default_layout = TextureAtlasLayout::from_grid(
    UVec2::splat(TILE_SIZE),
    TILE_SET_PLACEHOLDER_COLUMNS,
    TILE_SET_PLACEHOLDER_ROWS,
    None,
    None,
  );
  let default_texture_atlas_layout = layouts.add(default_layout);
  asset_collection.placeholder = AssetPack::new(asset_server.load(TILE_SET_PLACEHOLDER_PATH), default_texture_atlas_layout);

  // Detailed tile sets
  asset_collection.deep_water = tile_set_static(&asset_server, &mut layouts, TS_WATER_PATH);
  asset_collection.shallow_water = tile_set_default_animations(&asset_server, &mut layouts, TS_SHORE_PATH);
  asset_collection.land_dry_l1 = tile_set_default_animations(&asset_server, &mut layouts, TS_LAND_DRY_L1_PATH);
  asset_collection.land_dry_l2 = tile_set_static(&asset_server, &mut layouts, TS_LAND_DRY_L2_PATH);
  asset_collection.land_dry_l3 = tile_set_static(&asset_server, &mut layouts, TS_LAND_DRY_L3_PATH);
  asset_collection.land_moderate_l1 = tile_set_default_animations(&asset_server, &mut layouts, TS_LAND_MODERATE_L1_PATH);
  asset_collection.land_moderate_l2 = tile_set_static(&asset_server, &mut layouts, TS_LAND_MODERATE_L2_PATH);
  asset_collection.land_moderate_l3 = tile_set_static(&asset_server, &mut layouts, TS_LAND_MODERATE_L3_PATH);
  asset_collection.land_humid_l1 = tile_set_default_animations(&asset_server, &mut layouts, TS_LAND_HUMID_L1_PATH);
  asset_collection.land_humid_l2 = tile_set_static(&asset_server, &mut layouts, TS_LAND_HUMID_L2_PATH);
  asset_collection.land_humid_l3 = tile_set_static(&asset_server, &mut layouts, TS_LAND_HUMID_L3_PATH);

  // Objects: Trees
  let static_trees_layout = TextureAtlasLayout::from_grid(TREES_OBJ_SIZE, TREES_OBJ_COLUMNS, TREES_OBJ_ROWS, None, None);
  let static_trees_atlas_layout = layouts.add(static_trees_layout);
  asset_collection.objects.trees_dry.stat =
    AssetPack::new(asset_server.load(TREES_DRY_OBJ_PATH), static_trees_atlas_layout.clone());
  asset_collection.objects.trees_moderate.stat =
    AssetPack::new(asset_server.load(TREES_MODERATE_OBJ_PATH), static_trees_atlas_layout.clone());
  asset_collection.objects.trees_humid.stat =
    AssetPack::new(asset_server.load(TREES_HUMID_OBJ_PATH), static_trees_atlas_layout);

  // Objects: Paths
  asset_collection.objects.paths_water = path_assets_static(&asset_server, &mut layouts, 0);
  asset_collection.objects.paths_shore = path_assets_static(&asset_server, &mut layouts, 1);
  asset_collection.objects.paths_l1 = path_assets_static(&asset_server, &mut layouts, 2);
  asset_collection.objects.paths_l2 = path_assets_static(&asset_server, &mut layouts, 3);
  asset_collection.objects.paths_l3 = path_assets_static(&asset_server, &mut layouts, 4);

  // Objects: Terrain
  asset_collection.objects.water = object_assets_static(&asset_server, &mut layouts, WATER_DEEP_OBJ_PATH);
  asset_collection.objects.shore = object_assets_static(&asset_server, &mut layouts, WATER_SHALLOW_OBJ_PATH);
  asset_collection.objects.l1_dry = object_assets_static(&asset_server, &mut layouts, OBJ_L1_DRY_PATH);
  asset_collection.objects.l1_moderate = object_assets_static(&asset_server, &mut layouts, OBJ_L1_MODERATE_PATH);
  asset_collection.objects.l1_humid = object_assets_static(&asset_server, &mut layouts, OBJ_L1_HUMID_PATH);
  asset_collection.objects.l2_dry = object_assets_static(&asset_server, &mut layouts, OBJ_L2_DRY_PATH);
  asset_collection.objects.l2_moderate = object_assets_static(&asset_server, &mut layouts, OBJ_L2_MODERATE_PATH);
  asset_collection.objects.l2_humid = object_assets_static(&asset_server, &mut layouts, OBJ_L2_HUMID_PATH);
  asset_collection.objects.l3_dry = object_assets_static(&asset_server, &mut layouts, OBJ_L3_DRY_PATH);
  asset_collection.objects.l3_moderate = object_assets_static(&asset_server, &mut layouts, OBJ_L3_MODERATE_PATH);
  asset_collection.objects.l3_humid = object_assets_static(&asset_server, &mut layouts, OBJ_L3_HUMID_PATH);

  // Objects: Rule sets for wave function collapse
  let terrain_rules = terrain_rules(terrain_rule_set_handle, &mut terrain_rule_set_assets);
  let tile_type_rules = tile_type_rules(tile_type_rule_set_handle, &mut tile_type_rule_set_assets);
  asset_collection.objects.terrain_state_map = resolve_rules_to_terrain_states_map(terrain_rules, tile_type_rules);
}

fn tile_set_static(
  asset_server: &Res<AssetServer>,
  layout: &mut Assets<TextureAtlasLayout>,
  tile_set_path: &str,
) -> AssetCollection {
  let static_layout = TextureAtlasLayout::from_grid(
    UVec2::splat(TILE_SIZE),
    DEFAULT_STATIC_TILE_SET_COLUMNS,
    TILE_SET_ROWS,
    None,
    None,
  );
  let texture_atlas_layout = layout.add(static_layout);

  AssetCollection {
    stat: AssetPack::new(asset_server.load(tile_set_path.to_string()), texture_atlas_layout.clone()),
    anim: None,
    animated_tile_types: HashSet::new(),
  }
}

fn tile_set_default_animations(
  asset_server: &Res<AssetServer>,
  layout: &mut Assets<TextureAtlasLayout>,
  tile_set_path: &str,
) -> AssetCollection {
  let animated_tile_set_layout = TextureAtlasLayout::from_grid(
    UVec2::splat(TILE_SIZE),
    DEFAULT_ANIMATED_TILE_SET_COLUMNS,
    TILE_SET_ROWS,
    None,
    None,
  );
  let atlas_layout = layout.add(animated_tile_set_layout);

  AssetCollection {
    stat: AssetPack {
      texture: asset_server.load(tile_set_path.to_string()),
      texture_atlas_layout: atlas_layout.clone(),
      index_offset: DEFAULT_ANIMATED_TILE_SET_COLUMNS as usize,
    },
    anim: Some(AssetPack {
      texture: asset_server.load(tile_set_path.to_string()),
      texture_atlas_layout: atlas_layout,
      index_offset: DEFAULT_ANIMATED_TILE_SET_COLUMNS as usize,
    }),
    animated_tile_types: {
      let tile_types = [
        TileType::InnerCornerBottomLeft,
        TileType::InnerCornerBottomRight,
        TileType::InnerCornerTopLeft,
        TileType::InnerCornerTopRight,
        TileType::OuterCornerBottomLeft,
        TileType::OuterCornerBottomRight,
        TileType::OuterCornerTopLeft,
        TileType::OuterCornerTopRight,
        TileType::TopLeftToBottomRightBridge,
        TileType::TopRightToBottomLeftBridge,
        TileType::TopFill,
        TileType::BottomFill,
        TileType::RightFill,
        TileType::LeftFill,
        TileType::Single,
      ];
      insert(&tile_types)
    },
  }
}

fn insert(tile_types: &[TileType; 15]) -> HashSet<TileType> {
  let mut set = HashSet::new();
  for tile_type in tile_types {
    set.insert(*tile_type);
  }

  set
}

fn path_assets_static(
  asset_server: &Res<AssetServer>,
  layout: &mut Assets<TextureAtlasLayout>,
  offset: u32,
) -> AssetCollection {
  let static_layout = TextureAtlasLayout::from_grid(
    DEFAULT_OBJ_SIZE,
    PATHS_COLUMNS,
    1,
    None,
    Some(UVec2::new(0, offset * TILE_SIZE)),
  );
  let static_atlas_layout = layout.add(static_layout);

  AssetCollection {
    stat: AssetPack::new(asset_server.load(PATHS_PATH.to_string()), static_atlas_layout.clone()),
    anim: None,
    animated_tile_types: HashSet::new(),
  }
}

fn object_assets_static(
  asset_server: &Res<AssetServer>,
  layout: &mut Assets<TextureAtlasLayout>,
  tile_set_path: &str,
) -> AssetCollection {
  let static_layout = TextureAtlasLayout::from_grid(DEFAULT_OBJ_SIZE, DEFAULT_OBJ_COLUMNS, DEFAULT_OBJ_ROWS, None, None);
  let static_atlas_layout = layout.add(static_layout);

  AssetCollection {
    stat: AssetPack::new(asset_server.load(tile_set_path.to_string()), static_atlas_layout.clone()),
    anim: None,
    animated_tile_types: HashSet::new(),
  }
}

fn terrain_rules(
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
      "Found and removed [Any] terrain rule set with [{}] state(s) and will extend each of the other rule sets accordingly",
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
  }

  rule_sets
}

fn tile_type_rules(
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

// TODO: Consider adding some preprocessing logic that throws on invalid states e.g. if the specific terrain-tile type
//  combination x has y as a possible neighbour but y does not have x as a possible neighbour. I assume there'll be a
//  a few of those cases because the error rate for the WFC is too high.
/// Resolves the terrain rules and tile type rules into a single map that associates terrain types with tile types and
/// their possible states.
///
/// Note:
/// - [`TerrainType::Any`] is filtered out, as it is not a valid terrain type to be rendered and used to extend other
///   terrain types. This [`TerrainType`] causes panics if it is used later in the generation logic.
/// - [`TileType::Unknown`] is also filtered out, as it is not a valid tile type and is only used to signal
///   an error in the generation logic. This [`TileType`] will not cause panics but will be rendered as a bright,
///   single-coloured tile to indicate the error.
fn resolve_rules_to_terrain_states_map(
  terrain_rules: HashMap<TerrainType, Vec<TerrainState>>,
  tile_type_rules: HashMap<TileType, Vec<ObjectName>>,
) -> HashMap<TerrainType, HashMap<TileType, Vec<TerrainState>>> {
  let mut terrain_state_map: HashMap<TerrainType, HashMap<TileType, Vec<TerrainState>>> = HashMap::new();
  for terrain_type in TerrainType::iter().filter(|&t| t != TerrainType::Any) {
    let relevant_terrain_rules = terrain_rules
      .get(&terrain_type)
      .expect(format!("Failed to find rule set for [{:?}] terrain type", &terrain_type).as_str());
    let resolved_rules_for_terrain: HashMap<TileType, Vec<TerrainState>> = TileType::iter()
      .filter(|&t| t != TileType::Unknown)
      .map(|tile_type| {
        let all_rules_for_tile_type = tile_type_rules
          .get(&tile_type)
          .expect(&format!("Failed to find rule set for [{:?}] tile type", tile_type));
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
      terrain_type,
      resolved_rules_for_terrain
        .iter()
        .map(|(k, v)| (k, v.len()))
        .collect::<HashMap<&TileType, usize>>()
    );
    terrain_state_map.insert(terrain_type, resolved_rules_for_terrain);
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
