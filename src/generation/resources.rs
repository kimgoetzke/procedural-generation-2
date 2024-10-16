use crate::app_state::AppState;
use crate::constants::*;
use crate::coords::point::World;
use crate::coords::Point;
use crate::generation::lib::{ChunkComponent, TerrainType, TileType};
use bevy::app::{App, Plugin, Startup};
use bevy::asset::{AssetServer, Assets, Handle};
use bevy::log::*;
use bevy::math::UVec2;
use bevy::prelude::{Image, NextState, OnAdd, OnRemove, Query, Res, ResMut, Resource, TextureAtlasLayout, Trigger};
use bevy::utils::{HashMap, HashSet};

pub struct GenerationResourcesPlugin;

// TODO: Add game states and load assets prior to executing regular 'Startup' systems - see Bevy examples for this
impl Plugin for GenerationResourcesPlugin {
  fn build(&self, app: &mut App) {
    app
      .init_resource::<AssetPacksCollection>()
      .init_resource::<ChunkComponentIndex>()
      .observe(on_add_chunk_component_trigger)
      .observe(on_remove_chunk_component_trigger)
      .add_systems(Startup, initialise_asset_packs_system);
  }
}

#[derive(Resource, Default, Debug, Clone)]
pub struct AssetPacksCollection {
  pub placeholder: AssetPack,
  pub water: AssetPacks,
  pub shore: AssetPacks,
  pub sand: AssetPacks,
  pub sand_obj: AssetPacks,
  pub grass: AssetPacks,
  pub forest: AssetPacks,
  pub forest_obj: AssetPacks,
}

impl AssetPacksCollection {
  pub fn unpack_for_terrain(&self, terrain: TerrainType) -> &AssetPacks {
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
pub struct AssetPacks {
  pub stat: AssetPack,
  pub anim: Option<AssetPack>,
  pub animated_tile_types: HashSet<TileType>,
}

impl AssetPacks {
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

fn initialise_asset_packs_system(
  asset_server: Res<AssetServer>,
  mut layouts: ResMut<Assets<TextureAtlasLayout>>,
  mut asset_collection: ResMut<AssetPacksCollection>,
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
  asset_collection.forest_obj.stat = AssetPack::new(asset_server.load(FOREST_OBJ_PATH), static_trees_atlas_layout);

  // Objects: Stones
  let static_stones_layout = TextureAtlasLayout::from_grid(SAND_OBJ_SIZE, SAND_OBJ_COLUMNS, SAND_OBJ_ROWS, None, None);
  let static_stones_atlas_layout = layouts.add(static_stones_layout);
  asset_collection.sand_obj.stat = AssetPack::new(asset_server.load(SAND_OBJ_PATH), static_stones_atlas_layout);

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
) -> AssetPacks {
  let tile_set_layout = TextureAtlasLayout::from_grid(UVec2::splat(TILE_SIZE), columns, TILE_SET_ROWS, None, None);
  let texture_atlas_layout = layout.add(tile_set_layout);

  AssetPacks {
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
) -> AssetPacks {
  let animated_tile_set_layout = TextureAtlasLayout::from_grid(UVec2::splat(TILE_SIZE), columns, TILE_SET_ROWS, None, None);
  let atlas_layout = layout.add(animated_tile_set_layout);

  AssetPacks {
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
