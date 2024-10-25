use crate::constants::*;
use crate::coords::point::World;
use crate::coords::Point;
use crate::generation::lib::{ChunkComponent, TerrainType, TileType};
use crate::generation::object::lib::{Connection, ObjectName};
use crate::states::AppState;
use bevy::app::{App, Plugin, Startup};
use bevy::asset::{Asset, AssetServer, Assets, Handle};
use bevy::log::*;
use bevy::math::UVec2;
use bevy::prelude::{
  in_state, Commands, Image, IntoSystemConfigs, NextState, OnAdd, OnExit, OnRemove, Query, Res, ResMut, Resource,
  TextureAtlasLayout, Trigger, TypePath, Update,
};
use bevy::reflect::Reflect;
use bevy::utils::{HashMap, HashSet};
use bevy_common_assets::ron::RonAssetPlugin;
use std::fmt;
use std::fmt::{Display, Formatter};

pub struct GenerationResourcesPlugin;

impl Plugin for GenerationResourcesPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_plugins(RonAssetPlugin::<TerrainRuleSet>::new(&["terrain-ruleset.ron"]))
      .add_plugins(RonAssetPlugin::<TileTypeRuleSet>::new(&["type-ruleset.ron"]))
      .init_resource::<GenerationResourcesCollection>()
      .init_resource::<ChunkComponentIndex>()
      .observe(on_add_chunk_component_trigger)
      .observe(on_remove_chunk_component_trigger)
      .add_systems(Startup, load_rule_sets_system)
      .add_systems(Update, check_loading_state.run_if(in_state(AppState::Loading)))
      .add_systems(OnExit(AppState::Loading), initialise_resources_system);
  }
}

#[derive(serde::Deserialize, Asset, TypePath, Debug, Clone)]
struct TerrainRuleSet {
  terrain: TerrainType,
  states: Vec<TileState>,
}

#[derive(Resource, Default, Debug, Clone)]
struct TerrainRuleSetHandle(Vec<Handle<TerrainRuleSet>>);

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

#[derive(serde::Deserialize, Debug, Clone, Reflect)]
pub struct TileState {
  pub name: ObjectName,
  pub index: i32,
  pub permitted_neighbours: Vec<(Connection, Vec<(ObjectName, i32)>)>,
}

#[derive(serde::Deserialize, Asset, TypePath, Debug, Clone)]
struct TileTypeRuleSet {
  states: Vec<TileTypeState>,
}

#[derive(serde::Deserialize, Debug, Clone, Reflect)]
struct TileTypeState {
  pub tile_type: TileType,
  pub permitted_self: Vec<ObjectName>,
}

fn load_rule_sets_system(mut commands: Commands, asset_server: Res<AssetServer>) {
  let mut rule_set_handles = Vec::new();
  for i in 0..TerrainType::length() {
    let terrain_type = TerrainType::from(i);
    let path = format!("objects/{}.terrain-ruleset.ron", terrain_type.to_string().to_lowercase());
    let handle = asset_server.load(path);
    rule_set_handles.push(handle);
  }
  commands.insert_resource(TerrainRuleSetHandle(rule_set_handles));
  let handle = asset_server.load("objects/tile.type-ruleset.ron");
  commands.insert_resource(TileTypeRuleSetHandle(handle));
}

fn check_loading_state(
  asset_server: Res<AssetServer>,
  terrain_handles: Res<TerrainRuleSetHandle>,
  tile_type_handle: Res<TileTypeRuleSetHandle>,
  mut state: ResMut<NextState<AppState>>,
) {
  let tile_type_handle = &tile_type_handle.0;
  for terrain_handle in &terrain_handles.0 {
    if asset_server.get_load_state(terrain_handle) != Some(bevy::asset::LoadState::Loaded)
      || asset_server.get_load_state(tile_type_handle) != Some(bevy::asset::LoadState::Loaded)
    {
      debug_once!("Waiting for assets to load...");
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
  pub terrain_rule_sets: HashMap<TerrainType, Vec<TileState>>,
  pub tile_type_rule_sets: HashMap<TileType, Vec<ObjectName>>,
  pub water: AssetCollection,
  pub shore: AssetCollection,
  pub sand: AssetCollection,
  pub grass: AssetCollection,
  pub forest: AssetCollection,
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

  pub fn get_object_collection(&self, terrain: TerrainType) -> &AssetCollection {
    match terrain {
      TerrainType::Water => &self.objects.water,
      TerrainType::Shore => &self.objects.shore,
      TerrainType::Sand => &self.objects.sand,
      TerrainType::Grass => &self.objects.grass,
      TerrainType::Forest => &self.objects.forest,
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
  asset_collection.water = tile_set_assets_static(&asset_server, &mut layouts, TILE_SET_WATER_PATH);
  asset_collection.shore = tile_set_assets_with_default_animations(&asset_server, &mut layouts, TILE_SET_SHORE_PATH);
  asset_collection.sand = tile_set_assets_with_default_animations(&asset_server, &mut layouts, TILE_SET_SAND_PATH);
  asset_collection.grass = tile_set_assets_static(&asset_server, &mut layouts, TILE_SET_GRASS_PATH);
  asset_collection.forest = tile_set_assets_static(&asset_server, &mut layouts, TILE_SET_FOREST_PATH);

  // Objects: Trees
  let static_trees_layout = TextureAtlasLayout::from_grid(TREES_OBJ_SIZE, TREES_OBJ_COLUMNS, TREES_OBJ_ROWS, None, None);
  let static_trees_atlas_layout = layouts.add(static_trees_layout);
  asset_collection.objects.forest.stat = AssetPack::new(asset_server.load(TREES_OBJ_PATH), static_trees_atlas_layout);

  // Objects: Terrain
  asset_collection.objects.water = object_assets_static(&asset_server, &mut layouts, WATER_OBJ_PATH);
  asset_collection.objects.shore = object_assets_static(&asset_server, &mut layouts, SHORE_OBJ_PATH);
  asset_collection.objects.sand = object_assets_static(&asset_server, &mut layouts, SAND_OBJ_PATH);
  asset_collection.objects.grass = object_assets_static(&asset_server, &mut layouts, GRASS_OBJ_PATH);
  asset_collection.objects.forest = object_assets_static(&asset_server, &mut layouts, FOREST_OBJ_PATH);

  // Objects: Rule sets for wave function collapse
  let mut rule_sets = HashMap::new();
  for handle in terrain_rule_set_handle.0.iter() {
    if let Some(rule_set) = terrain_rule_set_assets.remove(handle) {
      debug!("Loaded: {}", rule_set);
      rule_sets.insert(rule_set.terrain, rule_set.states);
    }
  }
  asset_collection.objects.terrain_rule_sets = rule_sets;

  if let Some(rule_set) = tile_type_rule_set_assets.remove(&tile_type_rule_set_handle.0) {
    debug!("Loaded: Tile type rule set for {} tiles", rule_set.states.len());
    let mut rule_sets = HashMap::new();
    for state in rule_set.states {
      rule_sets.insert(state.tile_type, state.permitted_self);
    }
    asset_collection.objects.tile_type_rule_sets = rule_sets;
  }
}

fn tile_set_assets_static(
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

fn tile_set_assets_with_default_animations(
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

#[derive(Resource, Default)]
pub struct ChunkComponentIndex {
  map: HashMap<Point<World>, ChunkComponent>,
}

impl ChunkComponentIndex {
  pub fn get(&self, world: Point<World>) -> Option<&ChunkComponent> {
    if let Some(entity) = self.map.get(&world) {
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
  let cc = query.get(trigger.entity()).expect("Failed to get ChunkComponent");
  index.map.insert(cc.coords.world, cc.clone());
  trace!("ChunkComponentIndex <- Added ChunkComponent key w{:?}", cc.coords.world);
}

fn on_remove_chunk_component_trigger(
  trigger: Trigger<OnRemove, ChunkComponent>,
  query: Query<&ChunkComponent>,
  mut index: ResMut<ChunkComponentIndex>,
) {
  let cc = query.get(trigger.entity()).expect("Failed to get ChunkComponent");
  index.map.remove(&cc.coords.world);
  trace!(
    "ChunkComponentIndex -> Removed ChunkComponent with key w{:?}",
    cc.coords.world
  );
}
