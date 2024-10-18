use crate::app_state::AppState;
use crate::constants::*;
use crate::coords::point::World;
use crate::coords::Point;
use crate::generation::lib::{ChunkComponent, TerrainType, TileType};
use crate::generation::object::lib::{Connection, ObjectName};
use bevy::app::{App, Plugin, Startup};
use bevy::asset::{Asset, AssetServer, Assets, Handle};
use bevy::log::*;
use bevy::math::UVec2;
use bevy::prelude::{
  in_state, Commands, Image, IntoSystemConfigs, NextState, OnAdd, OnEnter, OnRemove, Query, Res, ResMut, Resource,
  TextureAtlasLayout, Trigger, TypePath, Update,
};
use bevy::utils::{HashMap, HashSet};
use bevy_common_assets::ron::RonAssetPlugin;
use std::fmt;
use std::fmt::{Display, Formatter};

pub struct GenerationResourcesPlugin;

impl Plugin for GenerationResourcesPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_plugins(RonAssetPlugin::<RuleSet>::new(&["ruleset.ron"]))
      .init_resource::<GenerationResourcesCollection>()
      .init_resource::<ChunkComponentIndex>()
      .observe(on_add_chunk_component_trigger)
      .observe(on_remove_chunk_component_trigger)
      .add_systems(Startup, pre_load_rule_sets_system)
      .add_systems(Update, check_loading_state.run_if(in_state(AppState::Loading)))
      .add_systems(OnEnter(AppState::Initialising), initialise_resources_system);
  }
}

#[derive(Resource, Default, Debug, Clone)]
struct RuleSetHandle(Vec<Handle<RuleSet>>);

#[derive(serde::Deserialize, Asset, TypePath, Debug, Clone)]
pub struct RuleSet {
  pub terrain: TerrainType,
  pub states: Vec<TileState>,
}

impl Default for RuleSet {
  fn default() -> Self {
    Self {
      terrain: TerrainType::Any,
      states: vec![],
    }
  }
}

impl Display for RuleSet {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "[{:?}] terrain rule set with {} states", self.terrain, self.states.len())
  }
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct TileState {
  pub name: ObjectName,
  pub index: i32,
  pub permitted_neighbours: Vec<(Connection, Vec<(ObjectName, i32)>)>,
}

fn pre_load_rule_sets_system(mut commands: Commands, asset_server: Res<AssetServer>) {
  // let sand = asset_server.load("objects/sand.ruleset.ron");
  let sand_path = asset_server.load("objects/sand-path.ruleset.ron");
  commands.insert_resource(RuleSetHandle(vec![sand_path]));
}

fn check_loading_state(asset_server: Res<AssetServer>, handles: Res<RuleSetHandle>, mut state: ResMut<NextState<AppState>>) {
  for handle in &handles.0 {
    if asset_server.get_load_state(handle) != Some(bevy::asset::LoadState::Loaded) {
      debug!("Waiting for assets to load...");
      return;
    }
  }

  debug!("Transitioning to [{:?}] state", AppState::Initialising);
  state.set(AppState::Initialising);
}

#[derive(Resource, Default, Debug, Clone)]
pub struct GenerationResourcesCollection {
  pub placeholder: AssetPack,
  pub water: AssetCollection,
  pub shore: AssetCollection,
  pub sand: AssetCollection,
  pub grass: AssetCollection,
  pub forest: AssetCollection,
  pub objects: ObjectResources,
}

#[derive(Resource, Default, Debug, Clone)]
pub struct ObjectResources {
  pub forest: AssetCollection,
  pub rule_sets: Vec<RuleSet>,
  pub sand: AssetCollection,
  pub path: AssetCollection,
}

impl GenerationResourcesCollection {
  pub fn get_terrain_collection(&self, terrain: TerrainType) -> &AssetCollection {
    match terrain {
      TerrainType::Water => &self.water,
      TerrainType::Shore => &self.shore,
      TerrainType::Sand => &self.sand,
      TerrainType::Grass => &self.grass,
      TerrainType::Forest => &self.forest,
      TerrainType::Any => panic!("You must not use TerrainType::Any when rendering tiles"),
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
  rule_set_handle: Res<RuleSetHandle>,
  mut rule_set_assets: ResMut<Assets<RuleSet>>,
  mut next_state: ResMut<NextState<AppState>>,
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
  asset_collection.water = tile_set_asset_packs_static(
    &asset_server,
    &mut layouts,
    TILE_SET_WATER_PATH,
    DEFAULT_STATIC_TILE_SET_COLUMNS,
  );
  asset_collection.shore = tile_set_asset_packs_with_default_animations(
    &asset_server,
    &mut layouts,
    TILE_SET_SHORE_PATH,
    DEFAULT_ANIMATED_TILE_SET_COLUMNS,
  );
  asset_collection.sand = tile_set_asset_packs_with_default_animations(
    &asset_server,
    &mut layouts,
    TILE_SET_SAND_PATH,
    DEFAULT_ANIMATED_TILE_SET_COLUMNS,
  );
  asset_collection.grass = tile_set_asset_packs_static(
    &asset_server,
    &mut layouts,
    TILE_SET_GRASS_PATH,
    DEFAULT_STATIC_TILE_SET_COLUMNS,
  );
  asset_collection.forest = tile_set_asset_packs_static(
    &asset_server,
    &mut layouts,
    TILE_SET_FOREST_PATH,
    DEFAULT_STATIC_TILE_SET_COLUMNS,
  );

  // Objects: Trees
  let static_trees_layout = TextureAtlasLayout::from_grid(FOREST_OBJ_SIZE, FOREST_OBJ_COLUMNS, FOREST_OBJ_ROWS, None, None);
  let static_trees_atlas_layout = layouts.add(static_trees_layout);
  asset_collection.objects.forest.stat = AssetPack::new(asset_server.load(FOREST_OBJ_PATH), static_trees_atlas_layout);

  // Objects: Stones
  let static_stones_layout = TextureAtlasLayout::from_grid(SAND_OBJ_SIZE, SAND_OBJ_COLUMNS, SAND_OBJ_ROWS, None, None);
  let static_stones_atlas_layout = layouts.add(static_stones_layout);
  asset_collection.objects.sand.stat = AssetPack::new(asset_server.load(SAND_OBJ_PATH), static_stones_atlas_layout);

  // Objects: Paths
  let static_paths_layout = TextureAtlasLayout::from_grid(PATHS_OBJ_SIZE, PATHS_OBJ_COLUMNS, PATHS_OBJ_ROWS, None, None);
  let static_paths_atlas_layout = layouts.add(static_paths_layout);
  asset_collection.objects.path.stat = AssetPack::new(asset_server.load(PATHS_OBJ_PATH), static_paths_atlas_layout);

  // Objects: Rule sets for wave function collapse
  let mut rule_sets = vec![];
  for handle in rule_set_handle.0.iter() {
    if let Some(rule_set) = rule_set_assets.remove(handle) {
      debug!("Loaded: {}", rule_set);
      rule_sets.push(rule_set);
    }
  }
  asset_collection.objects.rule_sets = rule_sets;

  debug!("Resources initialised, transitioning to [{:?}] state", AppState::Generating);
  next_state.set(AppState::Generating);
}

fn insert(tile_types: &[TileType; 15]) -> HashSet<TileType> {
  let mut set = HashSet::new();
  for tile_type in tile_types {
    set.insert(*tile_type);
  }

  set
}

fn tile_set_asset_packs_static(
  asset_server: &Res<AssetServer>,
  layout: &mut Assets<TextureAtlasLayout>,
  tile_set_path: &str,
  columns: u32,
) -> AssetCollection {
  let tile_set_layout = TextureAtlasLayout::from_grid(UVec2::splat(TILE_SIZE), columns, TILE_SET_ROWS, None, None);
  let texture_atlas_layout = layout.add(tile_set_layout);

  AssetCollection {
    stat: AssetPack::new(asset_server.load(tile_set_path.to_string()), texture_atlas_layout.clone()),
    anim: None,
    animated_tile_types: HashSet::new(),
  }
}

fn tile_set_asset_packs_with_default_animations(
  asset_server: &Res<AssetServer>,
  layout: &mut Assets<TextureAtlasLayout>,
  tile_set_path: &str,
  columns: u32,
) -> AssetCollection {
  let animated_tile_set_layout = TextureAtlasLayout::from_grid(UVec2::splat(TILE_SIZE), columns, TILE_SET_ROWS, None, None);
  let atlas_layout = layout.add(animated_tile_set_layout);

  AssetCollection {
    stat: AssetPack {
      texture: asset_server.load(tile_set_path.to_string()),
      texture_atlas_layout: atlas_layout.clone(),
      index_offset: columns as usize,
    },
    anim: Some(AssetPack {
      texture: asset_server.load(tile_set_path.to_string()),
      texture_atlas_layout: atlas_layout,
      index_offset: columns as usize,
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

#[derive(Resource, Default)]
pub struct ChunkComponentIndex {
  pub grid: HashMap<Point<World>, ChunkComponent>,
}

impl ChunkComponentIndex {
  pub fn get(&self, world: Point<World>) -> Option<&ChunkComponent> {
    if let Some(entity) = self.grid.get(&world) {
      Some(entity)
    } else {
      None
    }
  }
}

fn on_add_chunk_component_trigger(
  trigger: Trigger<OnAdd, ChunkComponent>,
  query: Query<&ChunkComponent>,
  mut index: ResMut<ChunkComponentIndex>,
) {
  let cc = query.get(trigger.entity()).unwrap();
  index.grid.insert(cc.coords.world, cc.clone());
  trace!("ChunkComponentIndex <- Added ChunkComponent key w{:?}", cc.coords.world);
}

fn on_remove_chunk_component_trigger(
  trigger: Trigger<OnRemove, ChunkComponent>,
  query: Query<&ChunkComponent>,
  mut index: ResMut<ChunkComponentIndex>,
) {
  let cc = query.get(trigger.entity()).unwrap();
  index.grid.remove(&cc.coords.world);
  trace!(
    "ChunkComponentIndex -> Removed ChunkComponent with key w{:?}",
    cc.coords.world
  );
}
