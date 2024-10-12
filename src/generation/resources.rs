use crate::constants::*;
use crate::coords::point::World;
use crate::coords::Point;
use crate::generation::lib::{ChunkComponent, TerrainType, TileType};
use bevy::app::{App, Plugin, Startup};
use bevy::asset::{AssetServer, Assets, Handle};
use bevy::log::*;
use bevy::math::UVec2;
use bevy::prelude::{Image, OnAdd, OnRemove, Query, Res, ResMut, Resource, TextureAtlasLayout, Trigger};
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
  pub default: AssetPack,
  pub water: AssetPacks,
  pub shore: AssetPacks,
  pub sand: AssetPacks,
  pub grass: AssetPacks,
  pub forest: AssetPacks,
  pub tree: AssetPacks,
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

#[derive(Default, Debug, Clone)]
pub struct AssetPack {
  pub texture: Handle<Image>,
  pub texture_atlas_layout: Handle<TextureAtlasLayout>,
}

fn initialise_asset_packs_system(
  asset_server: Res<AssetServer>,
  mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
  mut asset_collection: ResMut<AssetPacksCollection>,
) {
  let static_tile_set_layout = TextureAtlasLayout::from_grid(
    UVec2::splat(TILE_SIZE),
    STATIC_TILE_SET_COLUMNS,
    STATIC_TILE_SET_ROWS,
    None,
    None,
  );
  let static_tile_set_atlas_layout = texture_atlas_layouts.add(static_tile_set_layout);
  let animated_tile_set_layout = TextureAtlasLayout::from_grid(
    UVec2::splat(TILE_SIZE),
    ANIMATED_TILE_SET_COLUMNS,
    ANIMATED_TILE_SET_ROWS,
    None,
    None,
  );
  let animated_tile_set_atlas_layout = texture_atlas_layouts.add(animated_tile_set_layout);
  let default_layout = TextureAtlasLayout::from_grid(
    UVec2::splat(TILE_SIZE),
    TILE_SET_DEFAULT_COLUMNS,
    TILE_SET_DEFAULT_ROWS,
    None,
    None,
  );
  let default_texture_atlas_layout = texture_atlas_layouts.add(default_layout);
  let static_trees_layout = TextureAtlasLayout::from_grid(TREE_SIZE, TREES_COLUMNS, TREES_ROWS, None, None);
  let static_trees_atlas_layout = texture_atlas_layouts.add(static_trees_layout);

  asset_collection.default = AssetPack {
    texture: asset_server.load(TILE_SET_DEFAULT_PATH),
    texture_atlas_layout: default_texture_atlas_layout,
  };
  asset_collection.water.stat = AssetPack {
    texture: asset_server.load(STATIC_TILE_SET_WATER_PATH),
    texture_atlas_layout: static_tile_set_atlas_layout.clone(),
  };
  asset_collection.shore = asset_packs_with_default_anim(
    &asset_server,
    &static_tile_set_atlas_layout,
    animated_tile_set_atlas_layout.clone(),
    STATIC_TILE_SET_SHORE_PATH,
    ANIMATED_TILE_SET_SHORE_PATH,
  );
  asset_collection.sand = asset_packs_with_default_anim(
    &asset_server,
    &static_tile_set_atlas_layout,
    animated_tile_set_atlas_layout,
    STATIC_TILE_SET_SAND_PATH,
    ANIMATED_TILE_SET_SAND_PATH,
  );
  asset_collection.grass.stat = AssetPack {
    texture: asset_server.load(STATIC_TILE_SET_GRASS_PATH),
    texture_atlas_layout: static_tile_set_atlas_layout.clone(),
  };
  asset_collection.forest.stat = AssetPack {
    texture: asset_server.load(STATIC_TILE_SET_FOREST_PATH),
    texture_atlas_layout: static_tile_set_atlas_layout,
  };
  asset_collection.tree.stat = AssetPack {
    texture: asset_server.load(TREES_PATH),
    texture_atlas_layout: static_trees_atlas_layout,
  };
}

fn asset_packs_with_default_anim(
  asset_server: &Res<AssetServer>,
  static_layout: &Handle<TextureAtlasLayout>,
  animated_layout: Handle<TextureAtlasLayout>,
  static_path: &str,
  animated_path: &str,
) -> AssetPacks {
  AssetPacks {
    stat: AssetPack {
      texture: asset_server.load(static_path.to_string()),
      texture_atlas_layout: static_layout.clone(),
    },
    anim: Some(AssetPack {
      texture: asset_server.load(animated_path.to_string()),
      texture_atlas_layout: animated_layout,
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
