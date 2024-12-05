use crate::constants::*;
use crate::generation::lib::{TerrainType, TileType};
use crate::generation::object::lib::{Connection, ObjectName};
use crate::generation::resources::Climate;
use crate::states::AppState;
use bevy::app::{App, Plugin, Startup, Update};
use bevy::asset::{Asset, AssetServer, Assets, Handle, LoadState};
use bevy::log::*;
use bevy::math::UVec2;
use bevy::prelude::{
  in_state, Commands, Image, IntoSystemConfigs, NextState, OnExit, Reflect, Res, ResMut, Resource, TextureAtlasLayout,
  TypePath,
};
use bevy::utils::{HashMap, HashSet};
use bevy_common_assets::ron::RonAssetPlugin;
use std::fmt;
use std::fmt::{Display, Formatter};

pub struct GenerationResourcesCollectionPlugin;

impl Plugin for GenerationResourcesCollectionPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_plugins((
        RonAssetPlugin::<TerrainRuleSet>::new(&["terrain.ruleset.ron"]),
        RonAssetPlugin::<TileTypeRuleSet>::new(&["tile-type.ruleset.ron"]),
      ))
      .init_resource::<GenerationResourcesCollection>()
      .add_systems(Startup, load_rule_sets_system)
      .add_systems(Update, check_loading_state.run_if(in_state(AppState::Loading)))
      .add_systems(OnExit(AppState::Loading), initialise_resources_system);
  }
}

// --- Rules for wave function collapse -----------------------------------------------------

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

#[derive(serde::Deserialize, Debug, Clone, Reflect)]
pub struct TerrainState {
  pub name: ObjectName,
  pub index: i32,
  pub weight: i32,
  pub permitted_neighbours: Vec<(Connection, Vec<ObjectName>)>,
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
pub struct TileTypeState {
  pub tile_type: TileType,
  pub permitted_self: Vec<ObjectName>,
}

fn load_rule_sets_system(mut commands: Commands, asset_server: Res<AssetServer>) {
  let mut rule_set_handles = Vec::new();
  for i in 0..TerrainType::length() {
    let terrain_type = TerrainType::from(i);
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

fn check_loading_state(
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

// --- Universal asset resources for the generation process ----------------------------------

#[derive(Resource, Default, Debug, Clone)]
pub struct GenerationResourcesCollection {
  pub placeholder: AssetPack,
  pub deep_water: AssetCollection,
  pub shallow_water: AssetCollection,
  pub land_dry_l1: AssetCollection,
  pub land_dry_l2: AssetCollection,
  pub land_dry_l3: AssetCollection,
  pub land_moderate_l1: AssetCollection,
  pub land_moderate_l2: AssetCollection,
  pub land_moderate_l3: AssetCollection,
  pub land_humid_l1: AssetCollection,
  pub land_humid_l2: AssetCollection,
  pub land_humid_l3: AssetCollection,
  pub objects: ObjectResources,
}

#[derive(Resource, Default, Debug, Clone)]
pub struct ObjectResources {
  pub terrain_rules: HashMap<TerrainType, Vec<TerrainState>>,
  pub tile_type_rules: HashMap<TileType, Vec<ObjectName>>,
  pub water: AssetCollection,
  pub shore: AssetCollection,
  pub l1_dry: AssetCollection,
  pub l1_moderate: AssetCollection,
  pub l1_humid: AssetCollection,
  pub l2_dry: AssetCollection,
  pub l2_moderate: AssetCollection,
  pub l2_humid: AssetCollection,
  pub l3_dry: AssetCollection,
  pub l3_moderate: AssetCollection,
  pub l3_humid: AssetCollection,
  pub trees_dry: AssetCollection,
  pub trees_moderate: AssetCollection,
  pub trees_humid: AssetCollection,
}

impl GenerationResourcesCollection {
  pub fn get_terrain_collection(&self, terrain: TerrainType, climate: Climate) -> &AssetCollection {
    match (terrain, climate) {
      (TerrainType::DeepWater, _) => &self.deep_water,
      (TerrainType::ShallowWater, _) => &self.shallow_water,
      (TerrainType::Land1, Climate::Dry) => &self.land_dry_l1,
      (TerrainType::Land1, Climate::Moderate) => &self.land_moderate_l1,
      (TerrainType::Land1, Climate::Humid) => &self.land_humid_l1,
      (TerrainType::Land2, Climate::Dry) => &self.land_dry_l2,
      (TerrainType::Land2, Climate::Moderate) => &self.land_moderate_l2,
      (TerrainType::Land2, Climate::Humid) => &self.land_humid_l2,
      (TerrainType::Land3, Climate::Dry) => &self.land_dry_l3,
      (TerrainType::Land3, Climate::Moderate) => &self.land_moderate_l3,
      (TerrainType::Land3, Climate::Humid) => &self.land_humid_l3,
      (TerrainType::Any, _) => panic!("You must not use TerrainType::Any when rendering tiles"),
    }
  }

  pub fn get_object_collection(&self, terrain: TerrainType, climate: Climate, is_large_sprite: bool) -> &AssetCollection {
    match (terrain, climate, is_large_sprite) {
      (TerrainType::DeepWater, _, _) => &self.objects.water,
      (TerrainType::ShallowWater, _, _) => &self.objects.shore,
      (TerrainType::Land1, Climate::Dry, _) => &self.objects.l1_dry,
      (TerrainType::Land1, Climate::Moderate, _) => &self.objects.l1_moderate,
      (TerrainType::Land1, Climate::Humid, _) => &self.objects.l1_humid,
      (TerrainType::Land2, Climate::Dry, _) => &self.objects.l2_dry,
      (TerrainType::Land2, Climate::Moderate, _) => &self.objects.l2_moderate,
      (TerrainType::Land2, Climate::Humid, _) => &self.objects.l2_humid,
      (TerrainType::Land3, Climate::Dry, true) => &self.objects.trees_dry,
      (TerrainType::Land3, Climate::Moderate, true) => &self.objects.trees_moderate,
      (TerrainType::Land3, Climate::Humid, true) => &self.objects.trees_humid,
      (TerrainType::Land3, Climate::Dry, _) => &self.objects.l3_dry,
      (TerrainType::Land3, Climate::Moderate, _) => &self.objects.l3_moderate,
      (TerrainType::Land3, Climate::Humid, _) => &self.objects.l2_humid,
      (TerrainType::Any, _, _) => panic!("You must not use TerrainType::Any when rendering tiles"),
    }
  }
}

#[derive(Default, Debug, Clone)]
pub struct AssetCollection {
  pub stat: AssetPack,
  pub anim: Option<AssetPack>,
  pub animated_tile_types: HashSet<TileType>,
}

impl AssetCollection {
  pub fn index_offset(&self) -> usize {
    self.stat.index_offset
  }
}

#[derive(Debug, Clone)]
pub struct AssetPack {
  pub texture: Handle<Image>,
  pub texture_atlas_layout: Handle<TextureAtlasLayout>,
  pub index_offset: usize,
}

impl Default for AssetPack {
  fn default() -> Self {
    Self {
      texture: Handle::default(),
      texture_atlas_layout: Handle::default(),
      index_offset: 1,
    }
  }
}

impl AssetPack {
  pub fn new(texture: Handle<Image>, texture_atlas_layout: Handle<TextureAtlasLayout>) -> Self {
    Self {
      texture,
      texture_atlas_layout,
      index_offset: 1,
    }
  }
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
  asset_collection.objects.terrain_rules = terrain_rules(terrain_rule_set_handle, &mut terrain_rule_set_assets);
  asset_collection.objects.tile_type_rules = tile_type_rules(tile_type_rule_set_handle, &mut tile_type_rule_set_assets);
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
      "Found and removed [Any] terrain rule set with {} state(s) and will extend each of the other rule sets accordingly",
      any_rule_set.len()
    );
    for (terrain, states) in rule_sets.iter_mut() {
      states.splice(0..0, any_rule_set.iter().cloned());
      debug!(
        "Extended [{}] rule set by {}, it now has {} states",
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
    debug!("Loaded: Tile type rule set for {} tiles", rule_set.states.len());
    let mut rule_sets = HashMap::new();
    for state in rule_set.states {
      rule_sets.insert(state.tile_type, state.permitted_self);
    }
    return rule_sets;
  }

  HashMap::new()
}
