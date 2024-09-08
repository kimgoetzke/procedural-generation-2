use crate::settings::*;
use bevy::asset::{AssetServer, Assets};
use bevy::log::*;
use bevy::math::UVec2;
use bevy::prelude::{Handle, Image, Res, ResMut, TextureAtlasLayout};
use bevy::utils::{HashMap, HashSet};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Chunk {
  pub coords: Coords,
  pub center: Point,
  pub tiles: HashMap<Point, Tile>,
}

impl Chunk {
  pub fn new(world_location: Point, tiles: HashSet<Tile>) -> Self {
    let tiles_map = tiles.into_iter().map(|tile| (tile.coords.grid.clone(), tile)).collect();

    Self {
      center: Point::new(world_location.x + (CHUNK_SIZE / 2), world_location.y + (CHUNK_SIZE / 2)),
      coords: Coords::new_world(world_location),
      tiles: tiles_map,
    }
  }

  pub fn get_neighbour_tiles(&self, tile: &Tile) -> HashSet<&Tile> {
    let neighbour_points = [(-1, 0), (1, 0), (0, -1), (0, 1), (-1, 1), (1, 1), (-1, -1), (1, -1)];
    let mut neighbours = HashSet::new();
    for point in neighbour_points {
      let neighbour_pos = Point::new(tile.coords.grid.x + point.0, tile.coords.grid.y + point.1);
      if let Some(neighbour) = self.tiles.get(&neighbour_pos) {
        neighbours.insert(neighbour);
      }
    }
    neighbours
  }

  pub fn determine_tile_types(&mut self) {
    let neighbours_map: HashMap<Point, HashSet<Tile>> = {
      let tiles = &self.tiles;
      let mut map = HashMap::new();

      for tile_point in tiles.keys() {
        let mut neighbours = HashSet::new();
        let neighbour_points = [(-1, 0), (1, 0), (0, -1), (0, 1), (-1, 1), (1, 1), (-1, -1), (1, -1)];

        for point in neighbour_points.iter() {
          if let Some(neighbour) = tiles.get(&Point::new(tile_point.x + point.0, tile_point.y + point.1)) {
            neighbours.insert(neighbour.clone());
          }
        }

        map.insert(*tile_point, neighbours);
      }

      map
    };

    for tile in self.tiles.values_mut() {
      let neighbours = neighbours_map.get(&tile.coords.grid).unwrap();
      let mut same_type_count = 0;
      let mut different_type_count = 0;
      let mut neighbour_types = [false; 8];

      for (i, neighbour) in neighbours.iter().enumerate() {
        if neighbour.terrain == tile.terrain {
          same_type_count += 1;
          neighbour_types[i] = true;
        } else {
          different_type_count += 1;
        }
      }

      tile.tile_type = match (same_type_count, different_type_count) {
        (8, 0) => TileType::Fill,
        (5, 3) if neighbour_types[5] && neighbour_types[6] && neighbour_types[7] => TileType::BottomFill,
        (5, 3) if neighbour_types[0] && neighbour_types[1] && neighbour_types[2] => TileType::TopFill,
        (5, 3) if neighbour_types[0] && neighbour_types[3] && neighbour_types[5] => TileType::LeftFill,
        (5, 3) if neighbour_types[2] && neighbour_types[4] && neighbour_types[7] => TileType::RightFill,
        (3, 5) if neighbour_types[5] && neighbour_types[6] && neighbour_types[7] => TileType::InnerCornerBottomLeft,
        (3, 5) if neighbour_types[0] && neighbour_types[1] && neighbour_types[2] => TileType::InnerCornerTopRight,
        (3, 5) if neighbour_types[0] && neighbour_types[3] && neighbour_types[5] => TileType::InnerCornerTopLeft,
        (3, 5) if neighbour_types[2] && neighbour_types[4] && neighbour_types[7] => TileType::InnerCornerBottomRight,
        (7, 1) if !neighbour_types[5] => TileType::OuterCornerBottomLeft,
        (7, 1) if !neighbour_types[0] => TileType::OuterCornerTopLeft,
        (7, 1) if !neighbour_types[2] => TileType::OuterCornerTopRight,
        (7, 1) if !neighbour_types[7] => TileType::OuterCornerBottomRight,
        (6, 2) if !neighbour_types[0] && !neighbour_types[7] => TileType::TopLeftToBottomRightBridge,
        (6, 2) if !neighbour_types[2] && !neighbour_types[5] => TileType::TopRightToBottomLeftBridge,
        _ => TileType::Unknown,
      };

      trace!("{:?} => {:?}", tile, tile.tile_type);
    }
  }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Tile {
  pub coords: Coords,
  pub terrain: TerrainType,
  pub default_sprite_index: usize,
  pub layer: i32,
  pub tile_type: TileType,
}

impl Tile {
  pub fn new(grid_location: Point, terrain: TerrainType, sprite_index: usize, z_index: i32) -> Self {
    Self {
      coords: Coords::new_grid(grid_location, TILE_SIZE),
      terrain,
      default_sprite_index: sprite_index,
      layer: z_index,
      tile_type: TileType::Unknown,
    }
  }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
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
      grid: Point::new(world.x, world.y),
      world,
    }
  }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Point {
  pub x: i32,
  pub y: i32,
}

impl Point {
  pub fn new(x: i32, y: i32) -> Self {
    Self { x, y }
  }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum TerrainType {
  Water,
  Shore,
  Sand,
  Grass,
  Forest,
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

pub fn get_chunk_neighbour_points(chunk: Chunk) -> [Point; 8] {
  [
    Point::new(
      (chunk.coords.world.x - 1) * (CHUNK_SIZE - 1),
      chunk.coords.world.y * chunk.coords.world.y * (CHUNK_SIZE - 1),
    ),
    Point::new(
      (chunk.coords.world.x + 1) * (CHUNK_SIZE - 1),
      chunk.coords.world.y * chunk.coords.world.y * (CHUNK_SIZE - 1),
    ),
    Point::new(
      chunk.coords.world.x * (CHUNK_SIZE - 1),
      (chunk.coords.world.x - 1) * (CHUNK_SIZE - 1),
    ),
    Point::new(
      chunk.coords.world.x * (CHUNK_SIZE - 1),
      (chunk.coords.world.y + 1) * (CHUNK_SIZE - 1),
    ),
    Point::new(
      (chunk.coords.world.x - 1) * (CHUNK_SIZE - 1),
      (chunk.coords.world.y + 1) * (CHUNK_SIZE - 1),
    ),
    Point::new(
      (chunk.coords.world.x + 1) * (CHUNK_SIZE - 1),
      (chunk.coords.world.y + 1) * (CHUNK_SIZE - 1),
    ),
    Point::new(
      (chunk.coords.world.x - 1) * (CHUNK_SIZE - 1),
      (chunk.coords.world.x - 1) * (CHUNK_SIZE - 1),
    ),
    Point::new(
      (chunk.coords.world.x + 1) * (CHUNK_SIZE - 1),
      (chunk.coords.world.y - 1) * (CHUNK_SIZE - 1),
    ),
  ]
}

#[derive(Debug, Clone)]
pub struct AssetPacks {
  pub default: AssetPack,
  pub sand: AssetPack,
  pub grass: AssetPack,
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
    default: AssetPack {
      texture: asset_server.load(TILE_SET_DEFAULT_PATH),
      texture_atlas_layout: default_texture_atlas_layout,
    },
    sand: AssetPack {
      texture: asset_server.load(TILE_SET_SAND_PATH),
      texture_atlas_layout: texture_atlas_layout.clone(),
    },
    grass: AssetPack {
      texture: asset_server.load(TILE_SET_GRASS_PATH),
      texture_atlas_layout: texture_atlas_layout.clone(),
    },
  }
}
