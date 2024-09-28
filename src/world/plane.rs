use crate::constants::{BUFFER_SIZE, CHUNK_SIZE};
use crate::coords::Point;
use crate::resources::Settings;
use crate::world::neighbours::{NeighbourTile, NeighbourTiles};
use crate::world::terrain_type::TerrainType;
use crate::world::tile::{DraftTile, Tile};
use crate::world::tile_type::TileType;
use bevy::prelude::{debug, Res};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Plane {
  pub layer: Option<usize>,
  pub data: Vec<Vec<Option<Tile>>>,
}

impl Plane {
  pub fn new(draft_tiles: Vec<Vec<Option<DraftTile>>>, layer: Option<usize>, _settings: &Res<Settings>) -> Self {
    debug!("Creating a new [{:?}] layer plane", TerrainType::from(layer.unwrap_or(5)));
    let plane_data = determine_tile_types(&draft_tiles);
    let plane_data = resize_grid(plane_data);
    Self { data: plane_data, layer }
  }

  pub fn get_tile(&self, point: Point) -> Option<&Tile> {
    let i = point.x as usize;
    let j = point.y as usize;
    if i < self.data.len() && j < self.data[0].len() {
      self.data[i][j].as_ref()
    } else {
      None
    }
  }

  pub fn clear_tile(&mut self, point: Point) {
    self.data[point.x as usize][point.y as usize] = None;
  }

  pub fn get_neighbours(&self, of: &Tile) -> NeighbourTiles {
    let x = of.coords.chunk_grid.x;
    let y = of.coords.chunk_grid.y;
    let mut neighbours = NeighbourTiles::empty();

    for p in neighbour_points().iter() {
      let point = Point::new_chunk_grid(x + p.0, y + p.1);
      if let Some(neighbour) = self.get_tile(point) {
        let neighbour_tile = NeighbourTile::new(
          Point::new_chunk_grid(p.0, p.1),
          neighbour.terrain,
          neighbour.terrain == of.terrain || neighbour.layer > of.layer,
        );
        neighbours.put(neighbour_tile);
      } else {
        let neighbour_tile = NeighbourTile::default(Point::new_chunk_grid(p.0, p.1));
        neighbours.put(neighbour_tile);
      }
    }

    neighbours
  }
}

fn determine_tile_types(draft_tiles: &Vec<Vec<Option<DraftTile>>>) -> Vec<Vec<Option<Tile>>> {
  let y_len = draft_tiles.len();
  let x_len = draft_tiles[0].len();
  let mut final_tiles = vec![vec![None; y_len]; x_len];
  for y in 0..y_len {
    for x in 0..x_len {
      if let Some(draft_tile) = draft_tiles[x][y].as_ref() {
        if draft_tile.terrain == TerrainType::Water {
          let final_tile = Tile::from(draft_tile.clone(), TileType::Fill);
          final_tiles[draft_tile.coords.chunk_grid.x as usize][draft_tile.coords.chunk_grid.y as usize] = Some(final_tile);
        } else {
          let neighbour_tiles = get_neighbours(draft_tile, &draft_tiles);
          let same_neighbours_count = neighbour_tiles.count_same();
          let tile_type = determine_tile_type(neighbour_tiles, same_neighbours_count);
          let final_tile = Tile::from(draft_tile.clone(), tile_type);
          neighbour_tiles.print(&final_tile, same_neighbours_count);
          final_tiles[draft_tile.coords.chunk_grid.x as usize][draft_tile.coords.chunk_grid.y as usize] = Some(final_tile);
        }
      }
    }
  }

  final_tiles
}

// TODO: Consider refactoring this
// Consider generating terrain types for center and each corner of a tile using noise function
// and then use corner values to determine the tile type - may be slower though?
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
    6 if n.all_top_same() && n.all_right_same() && n.bottom_left.same => TileType::InnerCornerTopRight,
    6 if n.all_top_same() && n.all_left_same() && n.bottom_right.same => TileType::InnerCornerTopLeft,
    6 if n.all_bottom_same() && n.all_right_same() && n.top_left.same => TileType::InnerCornerBottomRight,
    6 if n.all_bottom_same() && n.all_left_same() && n.top_right.same => TileType::InnerCornerBottomLeft,
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
    6 if n.all_direction_bottom_left_same() & n.all_bottom_same() => TileType::InnerCornerBottomLeft,
    6 if n.all_direction_bottom_right_same() & n.all_bottom_same() => TileType::InnerCornerBottomRight,
    6 if n.all_direction_top_left_same() & n.all_top_same() => TileType::InnerCornerTopLeft,
    6 if n.all_direction_top_right_same() & n.all_top_same() => TileType::InnerCornerTopRight,
    6 if n.all_left_same() && n.all_direction_top_left_same() => TileType::InnerCornerTopLeft,
    6 if n.all_left_same() && n.all_direction_bottom_left_same() => TileType::InnerCornerBottomLeft,
    6 if n.all_right_same() && n.all_direction_top_right_same() => TileType::InnerCornerTopRight,
    6 if n.all_right_same() && n.all_direction_bottom_right_same() => TileType::InnerCornerBottomRight,
    6 if (!n.top.same && !n.bottom.same) | (!n.left.same && !n.right.same) => TileType::Single,
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
    5 => TileType::Single,
    4 if n.all_left_different() && !n.top.same => TileType::InnerCornerBottomRight,
    4 if n.all_left_different() && !n.bottom.same => TileType::InnerCornerTopRight,
    4 if n.all_right_different() && !n.top.same => TileType::InnerCornerBottomLeft,
    4 if n.all_right_different() && !n.bottom.same => TileType::InnerCornerTopLeft,
    4 if n.all_top_different() && !n.right.same => TileType::InnerCornerBottomLeft,
    4 if n.all_top_different() && !n.left.same => TileType::InnerCornerBottomRight,
    4 if n.all_bottom_different() && !n.left.same => TileType::InnerCornerTopRight,
    4 if n.all_bottom_different() && !n.right.same => TileType::InnerCornerTopLeft,
    4 if n.all_direction_top_left_different() && n.all_direction_bottom_right_same() => TileType::OuterCornerTopLeft,
    4 if n.all_direction_top_right_different() && n.all_direction_bottom_left_same() => TileType::OuterCornerTopRight,
    4 if n.all_direction_bottom_left_different() && n.all_direction_top_right_same() => TileType::OuterCornerBottomLeft,
    4 if n.all_direction_bottom_right_different() && n.all_direction_top_left_same() => TileType::OuterCornerBottomRight,
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
  let x = of.coords.chunk_grid.x;
  let y = of.coords.chunk_grid.y;
  let mut neighbours = NeighbourTiles::empty();

  for p in neighbour_points().iter() {
    if let Some(neighbour) = get_draft_tile(x + p.0, y + p.1 * -1, from) {
      let neighbour_tile = NeighbourTile::new(
        Point::new_abstract(p.0, p.1),
        neighbour.terrain,
        neighbour.terrain == of.terrain || neighbour.layer > of.layer,
      );
      neighbours.put(neighbour_tile);
    } else {
      let neighbour_tile = NeighbourTile::default(Point::new_abstract(p.0, p.1));
      neighbours.put(neighbour_tile);
    }
  }

  neighbours
}

fn neighbour_points() -> Vec<(i32, i32)> {
  vec![(-1, 1), (0, 1), (1, 1), (-1, 0), (1, 0), (-1, -1), (0, -1), (1, -1)]
}

fn get_draft_tile(x: i32, y: i32, from: &Vec<Vec<Option<DraftTile>>>) -> Option<&DraftTile> {
  if x >= 0 && x < from[0].len() as i32 && y >= 0 && y < from.len() as i32 {
    from[x as usize][y as usize].as_ref()
  } else {
    None
  }
}

/// Resizes the grid by cutting off the buffer size from each side of the grid. This is because the input data for
/// a plane is deliberately larger than the actual plane to allow for correct tile type determination on the edges.
/// For this to work, the `chunk_grid` `Coords` must be adjusted when creating a `Tile` from a `DraftTile`.
fn resize_grid(final_tiles: Vec<Vec<Option<Tile>>>) -> Vec<Vec<Option<Tile>>> {
  let cut_off = BUFFER_SIZE as usize;
  let mut cut_off_tiles = vec![vec![None; CHUNK_SIZE as usize]; CHUNK_SIZE as usize];

  for x in cut_off..final_tiles[0].len() - cut_off {
    for y in cut_off..final_tiles.len() - cut_off {
      cut_off_tiles[x - cut_off][y - cut_off] = final_tiles[x][y];
    }
  }

  cut_off_tiles
}
