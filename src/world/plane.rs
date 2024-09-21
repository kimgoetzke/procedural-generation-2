use crate::coords::Point;
use crate::resources::Settings;
use crate::world::neighbours::{NeighbourTile, NeighbourTiles};
use crate::world::terrain_type::TerrainType;
use crate::world::tile::{DraftTile, Tile};
use crate::world::tile_type::TileType;
use bevy::prelude::Res;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Plane {
  pub layer: Option<usize>,
  pub data: Vec<Vec<Option<Tile>>>,
}

impl Plane {
  pub fn new(draft_tiles: Vec<Vec<Option<DraftTile>>>, layer: Option<usize>, _settings: &Res<Settings>) -> Self {
    let plane_data = determine_tile_types(&draft_tiles);
    Self { data: plane_data, layer }
  }
}

pub fn determine_tile_types(draft_tiles: &Vec<Vec<Option<DraftTile>>>) -> Vec<Vec<Option<Tile>>> {
  let mut final_tiles = vec![vec![None; draft_tiles.len()]; draft_tiles[0].len()];
  for row in draft_tiles {
    for cell in row {
      if let Some(draft_tile) = cell {
        if draft_tile.terrain == TerrainType::Water {
          let tile = Tile::from(draft_tile.clone(), TileType::Fill);
          final_tiles[draft_tile.coords.chunk.x as usize][draft_tile.coords.chunk.y as usize] = Some(tile);
          continue;
        } else {
          let neighbour_tiles = get_neighbours(draft_tile, &draft_tiles);
          let same_neighbours_count = neighbour_tiles.count_same();
          let tile_type = determine_tile_type(neighbour_tiles, same_neighbours_count);
          let final_tile = Tile::from(draft_tile.clone(), tile_type);
          neighbour_tiles.print(&final_tile, same_neighbours_count);
          final_tiles[draft_tile.coords.chunk.x as usize][draft_tile.coords.chunk.y as usize] = Some(final_tile);
        }
      }
    }
  }

  final_tiles
}

fn determine_tile_type(n: NeighbourTiles, same_neighbours: usize) -> TileType {
  match same_neighbours {
    8 => TileType::Fill,
    7 if !n.top_left.same => TileType::OuterCornerTopLeft,
    7 if !n.top_right.same => TileType::OuterCornerTopRight,
    7 if !n.bottom_left.same => TileType::OuterCornerBottomLeft,
    7 if !n.bottom_right.same => TileType::OuterCornerBottomRight,
    7 if n.all_top_same() && !n.bottom.same => TileType::TopFill,
    7 if n.all_right_same() && !n.left.same => TileType::RightFill,
    7 if n.all_bottom_same() && !n.top.same => TileType::BottomFill,
    7 if n.all_left_same() && !n.right.same => TileType::LeftFill,
    6 if n.all_top_same() && (n.bottom_left.same || n.bottom_right.same) && !n.bottom.same => TileType::TopFill,
    6 if n.all_right_same() && (n.top_left.same || n.bottom_left.same) && !n.left.same => TileType::RightFill,
    6 if n.all_bottom_same() && (n.top_left.same || n.top_right.same) && !n.top.same => TileType::BottomFill,
    6 if n.all_left_same() && (n.top_right.same || n.bottom_right.same) && !n.right.same => TileType::LeftFill,
    6 if !n.top_right.same && !n.bottom_left.same => TileType::TopLeftToBottomRightBridge,
    6 if !n.top_left.same && !n.bottom_right.same => TileType::TopRightToBottomLeftBridge,
    6 if n.all_top_same() && n.all_sides_same() => TileType::TopFill,
    6 if n.all_right_same() && n.all_sides_same() => TileType::RightFill,
    6 if n.all_bottom_same() && n.all_sides_same() => TileType::BottomFill,
    6 if n.all_left_same() && n.all_sides_same() => TileType::LeftFill,
    5 if n.all_top_same() && n.left.same && n.right.same => TileType::TopFill,
    5 if n.all_right_same() && n.top.same && n.bottom.same => TileType::RightFill,
    5 if n.all_bottom_same() && n.left.same && n.right.same => TileType::BottomFill,
    5 if n.all_left_same() && n.top.same && n.bottom.same => TileType::LeftFill,
    5 if n.all_top_same() && n.all_right_same() => TileType::InnerCornerTopRight,
    5 if n.all_top_same() && n.all_left_same() => TileType::InnerCornerTopLeft,
    5 if n.all_bottom_same() && n.all_right_same() => TileType::InnerCornerBottomRight,
    5 if n.all_bottom_same() && n.all_left_same() => TileType::InnerCornerBottomLeft,
    5 if n.all_direction_top_left_same() => TileType::InnerCornerTopLeft,
    5 if n.all_direction_top_right_same() => TileType::InnerCornerTopRight,
    5 if n.all_direction_bottom_left_same() => TileType::InnerCornerBottomLeft,
    5 if n.all_direction_bottom_right_same() => TileType::InnerCornerBottomRight,
    4 if n.all_left_different() && !n.top.same => TileType::InnerCornerBottomRight,
    4 if n.all_left_different() && !n.bottom.same => TileType::InnerCornerTopRight,
    4 if n.all_right_different() && !n.top.same => TileType::InnerCornerBottomLeft,
    4 if n.all_right_different() && !n.bottom.same => TileType::InnerCornerTopLeft,
    4 if n.all_top_different() && !n.right.same => TileType::InnerCornerBottomLeft,
    4 if n.all_top_different() && !n.left.same => TileType::InnerCornerBottomRight,
    4 if n.all_bottom_different() && !n.left.same => TileType::InnerCornerTopRight,
    4 if n.all_bottom_different() && !n.right.same => TileType::InnerCornerTopLeft,
    4 if n.all_direction_top_left_different() => TileType::OuterCornerTopLeft,
    4 if n.all_direction_top_right_different() => TileType::OuterCornerTopRight,
    4 if n.all_direction_bottom_left_different() => TileType::OuterCornerBottomLeft,
    4 if n.all_direction_bottom_right_different() => TileType::OuterCornerBottomRight,
    4 if n.all_direction_top_left_same() => TileType::InnerCornerTopLeft,
    4 if n.all_direction_top_right_same() => TileType::InnerCornerTopRight,
    4 if n.all_direction_bottom_left_same() => TileType::InnerCornerBottomLeft,
    4 if n.all_direction_bottom_right_same() => TileType::InnerCornerBottomRight,
    4 => TileType::Single,
    3 if n.all_direction_top_left_same() => TileType::InnerCornerTopLeft,
    3 if n.all_direction_top_right_same() => TileType::InnerCornerTopRight,
    3 if n.all_direction_bottom_left_same() => TileType::InnerCornerBottomLeft,
    3 if n.all_direction_bottom_right_same() => TileType::InnerCornerBottomRight,
    3 | 2 | 1 | 0 => TileType::Single,
    _ => TileType::Unknown,
  }
}

fn get_neighbours(of: &DraftTile, from: &Vec<Vec<Option<DraftTile>>>) -> NeighbourTiles {
  let x = of.coords.chunk.x;
  let y = of.coords.chunk.y;
  let neighbour_points = vec![(-1, 1), (0, 1), (1, 1), (-1, 0), (1, 0), (-1, -1), (0, -1), (1, -1)];
  let mut neighbours = NeighbourTiles::empty();

  for p in neighbour_points.iter() {
    if let Some(neighbour) = get_draft_tile(x + p.0, y + p.1, from) {
      let neighbour_tile = NeighbourTile::new(
        Point::new(p.0, p.1),
        neighbour.terrain,
        neighbour.terrain == of.terrain || neighbour.layer > of.layer,
      );
      neighbours.put(neighbour_tile);
    } else {
      let neighbour_tile = NeighbourTile::default(Point::new(p.0, p.1));
      neighbours.put(neighbour_tile);
    }
  }

  neighbours
}

pub fn get_draft_tile(x: i32, y: i32, from: &Vec<Vec<Option<DraftTile>>>) -> Option<&DraftTile> {
  let i = x as usize;
  let j = y as usize;
  if i < from.len() && j < from[0].len() {
    from[i][j].as_ref()
  } else {
    None
  }
}
