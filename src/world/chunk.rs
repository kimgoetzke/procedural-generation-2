use crate::resources::Settings;
use crate::settings::CHUNK_SIZE;
use crate::world::neighbours::{NeighbourTile, NeighbourTiles};
use crate::world::shared::{Coords, Point, TerrainType, TileType};
use crate::world::tile::{DraftTile, Tile};
use bevy::log::warn;
use bevy::prelude::Res;
use bevy::utils::HashMap;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct DraftChunk {
  pub coords: Coords,
  center: Point,
  draft_tiles: Vec<DraftTile>,
}

impl DraftChunk {
  pub fn new(world_location: Point, draft_tiles: Vec<DraftTile>) -> Self {
    Self {
      center: Point::new(world_location.x + (CHUNK_SIZE / 2), world_location.y + (CHUNK_SIZE / 2)),
      coords: Coords::new_world(world_location),
      draft_tiles,
    }
  }

  pub fn to_chunk(self, settings: &Res<Settings>) -> Chunk {
    let tiles = determine_tile_types(self.draft_tiles, settings);
    Chunk {
      coords: self.coords,
      center: self.center,
      tiles,
    }
  }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Chunk {
  pub coords: Coords,
  pub center: Point,
  pub tiles: Vec<Tile>,
}

fn determine_tile_types(draft_tiles: Vec<DraftTile>, settings: &Res<Settings>) -> Vec<Tile> {
  let neighbours_map: HashMap<Point, NeighbourTiles> = {
    let tiles = &draft_tiles;
    let mut map = HashMap::new();

    for draft_tile in &draft_tiles {
      let mut neighbours = NeighbourTiles::empty();
      let neighbour_points = vec![(-1, 1), (0, 1), (1, 1), (-1, 0), (1, 0), (-1, -1), (0, -1), (1, -1)];

      for point in neighbour_points.iter() {
        if let Some(neighbour) = tiles
          .iter()
          .find(|t| t.coords.grid == Point::new(draft_tile.coords.grid.x + point.0, draft_tile.coords.grid.y + point.1))
        {
          neighbours.put(NeighbourTile::new(
            Point::new(point.0, point.1),
            neighbour.terrain,
            neighbour.terrain == draft_tile.terrain,
          ));
        } else {
          neighbours.put(NeighbourTile::default(Point::new(point.0, point.1)));
        }
      }

      map.insert(draft_tile.coords.grid.clone(), neighbours);
    }

    map
  };

  let mut final_tiles = Vec::new();
  for tile in draft_tiles.iter() {
    if tile.terrain == TerrainType::Water {
      final_tiles.push(Tile::from(tile.clone(), TileType::Fill));
      continue;
    }

    let ns = neighbours_map.get(&tile.coords.grid).unwrap();
    let same_neighbours = ns.count_same();

    let tile_type = match same_neighbours {
      8 => TileType::Fill,
      7 if !ns.top_left.same => TileType::OuterCornerTopLeft,
      7 if !ns.top_right.same => TileType::OuterCornerTopRight,
      7 if !ns.bottom_left.same => TileType::OuterCornerBottomLeft,
      7 if !ns.bottom_right.same => TileType::OuterCornerBottomRight,
      6 if ns.all_right_same() && (ns.top_left.same || ns.bottom_left.same) && !ns.left.same => TileType::RightFill,
      6 if ns.all_left_same() && (ns.top_right.same || ns.bottom_right.same) && !ns.right.same => TileType::LeftFill,
      6 if ns.all_bottom_same() && (ns.top_left.same || ns.top_right.same) && !ns.top.same => TileType::BottomFill,
      6 if ns.all_top_same() && (ns.bottom_left.same || ns.bottom_right.same) && !ns.bottom.same => TileType::TopFill,
      6 if !ns.top_right.same && !ns.bottom_left.same => TileType::TopLeftToBottomRightBridge,
      6 if !ns.top_left.same && !ns.bottom_right.same => TileType::TopRightToBottomLeftBridge,
      5 if ns.all_top_same() && ns.left.same && ns.right.same => TileType::TopFill,
      5 if ns.all_bottom_same() && ns.left.same && ns.right.same => TileType::BottomFill,
      5 if ns.all_left_same() && ns.top.same && ns.bottom.same => TileType::LeftFill,
      5 if ns.all_right_same() && ns.top.same && ns.bottom.same => TileType::RightFill,
      5 if ns.all_top_same() && ns.all_right_same() => TileType::InnerCornerTopRight,
      5 if ns.all_top_same() && ns.all_left_same() => TileType::InnerCornerTopLeft,
      5 if ns.all_bottom_same() && ns.all_right_same() => TileType::InnerCornerBottomRight,
      5 if ns.all_bottom_same() && ns.all_left_same() => TileType::InnerCornerBottomLeft,
      4 if ns.all_left_different() && !ns.top.same => TileType::InnerCornerBottomRight,
      4 if ns.all_left_different() && !ns.bottom.same => TileType::InnerCornerTopRight,
      4 if ns.all_right_different() && !ns.top.same => TileType::InnerCornerBottomLeft,
      4 if ns.all_right_different() && !ns.bottom.same => TileType::InnerCornerTopLeft,
      4 if ns.all_top_different() && !ns.right.same => TileType::InnerCornerBottomLeft,
      4 if ns.all_top_different() && !ns.left.same => TileType::InnerCornerBottomRight,
      4 if ns.all_bottom_different() && !ns.left.same => TileType::InnerCornerTopRight,
      4 if ns.all_bottom_different() && !ns.right.same => TileType::InnerCornerTopLeft,
      4 if ns.all_direction_top_left_different() => TileType::OuterCornerTopLeft,
      4 if ns.all_direction_top_right_different() => TileType::OuterCornerTopRight,
      4 if ns.all_direction_bottom_left_different() => TileType::OuterCornerBottomLeft,
      4 if ns.all_direction_bottom_right_different() => TileType::OuterCornerBottomRight,
      3 if ns.all_direction_top_left_same() => TileType::InnerCornerTopLeft,
      3 if ns.all_direction_top_right_same() => TileType::InnerCornerTopRight,
      3 if ns.all_direction_bottom_left_same() => TileType::InnerCornerBottomLeft,
      3 if ns.all_direction_bottom_right_same() => TileType::InnerCornerBottomRight,
      2 => TileType::Fill,
      1 if ns.top.same && ns.right.same && ns.bottom.same && ns.left.same => TileType::Fill,
      1 if ns.top.same => TileType::Unknown,
      1 => TileType::Unknown,
      _ => TileType::Unknown,
    };

    let mut final_tile = Tile::from(tile.clone(), tile_type);

    if settings.permit_tile_layer_adjustments {
      if final_tile.tile_type == TileType::Fill && ns.count_same() != 8 {
        final_tile.move_to_lower_terrain_layer();
        warn!("Adjusted before finalising: {:?}...", tile);
      } else if final_tile.tile_type == TileType::Unknown && ns.count_same() == 4 {
        final_tile.move_to_lower_terrain_layer();
        final_tile.tile_type = TileType::Fill;
        warn!("Adjusted before finalising: {:?}...", tile);
      } else if final_tile.tile_type == TileType::Unknown
        && (ns.all_left_same() || ns.all_right_same() || ns.all_top_same() || ns.all_bottom_same())
        && ns.count_same() == 3
      {
        final_tile.move_to_lower_terrain_layer();
        final_tile.tile_type = TileType::Fill;
        warn!("Adjusted before finalising: {:?}...", tile);
      }
    }

    ns.print(&final_tile, same_neighbours);
    final_tiles.push(final_tile);
  }

  final_tiles
}

#[allow(dead_code)]
pub fn get_neighbour_world_points(coords: &Coords, adjustment: i32) -> [Point; 8] {
  let point = coords.world;
  let adjustment = adjustment - 1;
  [
    Point::new(point.x - adjustment, point.y + adjustment),
    Point::new(point.x, point.y + adjustment),
    Point::new(point.x + adjustment, point.y + adjustment),
    Point::new(point.x - adjustment, point.y),
    Point::new(point.x + adjustment, point.y),
    Point::new(point.x - adjustment, point.y - adjustment),
    Point::new(point.x, point.y - adjustment),
    Point::new(point.x + adjustment, point.y - adjustment),
  ]
}
