use crate::settings::*;
use crate::world::tile::Tile;
use bevy::asset::{AssetServer, Assets};
use bevy::math::UVec2;
use bevy::prelude::{Handle, Image, Res, ResMut, TextureAtlasLayout};
use std::cmp::PartialOrd;
use std::fmt;

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Coords {
  pub world: Point,
  pub grid: Point,
}

impl Coords {
  pub fn new_grid(grid: Point, tile_size: u32) -> Self {
    Self {
      world: Point::new(grid.x * tile_size as i32, grid.y * tile_size as i32),
      grid,
    }
  }

  pub fn new_world(world: Point) -> Self {
    Self {
      grid: Point::new(world.x / CHUNK_SIZE, world.y / CHUNK_SIZE),
      world,
    }
  }
}

impl fmt::Debug for Coords {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "[w{}, g{}]", self.world, self.grid)
  }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Point {
  pub x: i32,
  pub y: i32,
}

impl fmt::Debug for Point {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "({}, {})", self.x, self.y)
  }
}

impl fmt::Display for Point {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "({}, {})", self.x, self.y)
  }
}

impl Point {
  pub fn new(x: i32, y: i32) -> Self {
    Self { x, y }
  }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Hash)]
pub enum TerrainType {
  Water,
  Shore,
  Sand,
  Grass,
  Forest,
  Any,
}

impl Default for TerrainType {
  fn default() -> Self {
    TerrainType::Any
  }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum TileType {
  Fill,
  InnerCornerBottomLeft,
  InnerCornerBottomRight,
  InnerCornerTopLeft,
  InnerCornerTopRight,
  OuterCornerBottomLeft,
  OuterCornerBottomRight,
  OuterCornerTopLeft,
  OuterCornerTopRight,
  TopLeftToBottomRightBridge,
  TopRightToBottomLeftBridge,
  TopFill,
  BottomFill,
  RightFill,
  LeftFill,
  Empty,
  Unknown,
}

pub fn get_sprite_index(tile: &Tile) -> usize {
  match tile.tile_type {
    TileType::Fill => FILL,
    TileType::InnerCornerBottomLeft => INNER_CORNER_BOTTOM_LEFT,
    TileType::InnerCornerBottomRight => INNER_CORNER_BOTTOM_RIGHT,
    TileType::InnerCornerTopLeft => INNER_CORNER_TOP_LEFT,
    TileType::InnerCornerTopRight => INNER_CORNER_TOP_RIGHT,
    TileType::OuterCornerBottomLeft => OUTER_CORNER_BOTTOM_LEFT,
    TileType::OuterCornerBottomRight => OUTER_CORNER_BOTTOM_RIGHT,
    TileType::OuterCornerTopLeft => OUTER_CORNER_TOP_LEFT,
    TileType::OuterCornerTopRight => OUTER_CORNER_TOP_RIGHT,
    TileType::TopLeftToBottomRightBridge => TOP_LEFT_TO_BOTTOM_RIGHT_BRIDGE,
    TileType::TopRightToBottomLeftBridge => TOP_RIGHT_TO_BOTTOM_LEFT_BRIDGE,
    TileType::TopFill => TOP_FILL,
    TileType::BottomFill => BOTTOM_FILL,
    TileType::RightFill => RIGHT_FILL,
    TileType::LeftFill => LEFT_FILL,
    _ => ERROR,
  }
}

#[derive(Debug, Clone)]
pub struct AssetPacks {
  pub font: Handle<Font>,
  pub default: AssetPack,
  pub sand: AssetPack,
}

#[derive(Debug, Clone)]
pub struct AssetPack {
  pub texture: Handle<Image>,
  pub texture_atlas_layout: Handle<TextureAtlasLayout>,
}

pub fn get_asset_packs(
  asset_server: &Res<AssetServer>,
  texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
) -> AssetPacks {
  let layout = TextureAtlasLayout::from_grid(UVec2::splat(TILE_SIZE), TILE_SET_COLUMNS, TILE_SET_ROWS, None, None);
  let texture_atlas_layout = texture_atlas_layouts.add(layout);
  let default_layout = TextureAtlasLayout::from_grid(
    UVec2::splat(TILE_SIZE),
    TILE_SET_DEFAULT_COLUMNS,
    TILE_SET_DEFAULT_ROWS,
    None,
    None,
  );
  let default_texture_atlas_layout = texture_atlas_layouts.add(default_layout);

  AssetPacks {
    font: asset_server.load(DEFAULT_FONT),
    default: AssetPack {
      texture: asset_server.load(TILE_SET_DEFAULT_PATH),
      texture_atlas_layout: default_texture_atlas_layout,
    },
    sand: AssetPack {
      texture: asset_server.load(TILE_SET_SAND_PATH),
      texture_atlas_layout: texture_atlas_layout.clone(),
    },
    // grass: AssetPack {
    //   texture: asset_server.load(TILE_SET_GRASS_PATH),
    //   texture_atlas_layout: texture_atlas_layout.clone(),
    // },
  }
}
