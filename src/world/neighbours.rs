use crate::world::shared::{Point, TerrainType};
use crate::world::tile::Tile;
use bevy::log::*;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct NeighbourTile {
  pub direction: Point,
  pub terrain: TerrainType,
  pub is_same: bool,
  pub is_any: bool,
}

impl NeighbourTile {
  pub fn default(direction: Point) -> Self {
    Self {
      direction,
      terrain: TerrainType::Any,
      is_same: false,
      is_any: true,
    }
  }

  pub(crate) fn new(direction: Point, terrain_type: TerrainType, is_same: bool) -> NeighbourTile {
    NeighbourTile {
      direction,
      terrain: terrain_type,
      is_same,
      is_any: terrain_type == TerrainType::Any,
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub struct NeighbourTiles {
  pub top_left: NeighbourTile,
  pub top: NeighbourTile,
  pub top_right: NeighbourTile,
  pub left: NeighbourTile,
  pub right: NeighbourTile,
  pub bottom_left: NeighbourTile,
  pub bottom: NeighbourTile,
  pub bottom_right: NeighbourTile,
}

#[allow(dead_code)]
impl NeighbourTiles {
  pub(crate) fn empty() -> Self {
    Self {
      top_left: NeighbourTile::default(Point::new(-1, 1)),
      top: NeighbourTile::default(Point::new(0, 1)),
      top_right: NeighbourTile::default(Point::new(1, 1)),
      left: NeighbourTile::default(Point::new(-1, 0)),
      right: NeighbourTile::default(Point::new(1, 0)),
      bottom_left: NeighbourTile::default(Point::new(-1, -1)),
      bottom: NeighbourTile::default(Point::new(0, -1)),
      bottom_right: NeighbourTile::default(Point::new(1, -1)),
    }
  }

  fn put_all(ordered_neighbours: Vec<NeighbourTile>) -> Self {
    let find_tile = |x, y| {
      ordered_neighbours
        .iter()
        .find(|n| n.direction.x == x && n.direction.y == y)
        .cloned()
        .unwrap()
    };

    Self {
      top_left: find_tile(-1, 1),
      top: find_tile(0, 1),
      top_right: find_tile(1, 1),
      left: find_tile(-1, 0),
      right: find_tile(1, 0),
      bottom_left: find_tile(-1, -1),
      bottom: find_tile(0, -1),
      bottom_right: find_tile(1, -1),
    }
  }

  pub fn all_direction_top_left_same(&self) -> bool {
    self.top_left.is_same && self.top.is_same && self.left.is_same
  }

  pub fn all_direction_top_right_same(&self) -> bool {
    self.top.is_same && self.top_right.is_same && self.right.is_same
  }

  pub fn all_direction_bottom_left_same(&self) -> bool {
    self.left.is_same && self.bottom.is_same && self.bottom_left.is_same
  }

  pub fn all_direction_bottom_right_same(&self) -> bool {
    self.bottom.is_same && self.bottom_right.is_same && self.right.is_same
  }

  pub fn all_direction_top_left_different(&self) -> bool {
    !self.top_left.is_same && !self.top.is_same && !self.left.is_same
  }

  pub fn all_direction_top_right_different(&self) -> bool {
    !self.top.is_same && !self.top_right.is_same && !self.right.is_same
  }

  pub fn all_direction_bottom_left_different(&self) -> bool {
    !self.left.is_same && !self.bottom.is_same && !self.bottom_left.is_same
  }

  pub fn all_direction_bottom_right_different(&self) -> bool {
    !self.bottom.is_same && !self.bottom_right.is_same && !self.right.is_same
  }

  pub fn all_top_same(&self) -> bool {
    self.top_left.is_same && self.top.is_same && self.top_right.is_same
  }

  pub fn all_top_different(&self) -> bool {
    !self.top_left.is_same && !self.top.is_same && !self.top_right.is_same
  }

  pub fn top_same(&self, expected: usize) -> bool {
    [self.top_left, self.top, self.top_right]
      .iter()
      .filter(|&&tile| tile.is_same)
      .count()
      == expected
  }

  pub fn all_bottom_same(&self) -> bool {
    self.bottom_left.is_same && self.bottom.is_same && self.bottom_right.is_same
  }

  pub fn all_bottom_different(&self) -> bool {
    !self.bottom_left.is_same && !self.bottom.is_same && !self.bottom_right.is_same
  }

  pub fn bottom_same(&self, expected: usize) -> bool {
    [self.bottom_left, self.bottom, self.bottom_right]
      .iter()
      .filter(|&&tile| tile.is_same)
      .count()
      == expected
  }

  pub fn all_left_same(&self) -> bool {
    self.top_left.is_same && self.left.is_same && self.bottom_left.is_same
  }

  pub fn all_left_different(&self) -> bool {
    !self.top_left.is_same && !self.left.is_same && !self.bottom_left.is_same
  }

  pub fn left_same(&self, expected: usize) -> bool {
    [self.top_left, self.left, self.bottom_left]
      .iter()
      .filter(|&&tile| tile.is_same)
      .count()
      == expected
  }

  pub fn all_right_same(&self) -> bool {
    self.top_right.is_same && self.right.is_same && self.bottom_right.is_same
  }

  pub fn all_right_different(&self) -> bool {
    !self.top_right.is_same && !self.right.is_same && !self.bottom_right.is_same
  }

  pub fn right_same(&self, expected: usize) -> bool {
    [self.top_right, self.right, self.bottom_right]
      .iter()
      .filter(|&&tile| tile.is_same)
      .count()
      == expected
  }

  pub fn put(&mut self, tile: NeighbourTile) {
    match (tile.direction.x, tile.direction.y) {
      (-1, 1) => self.top_left = tile,
      (0, 1) => self.top = tile,
      (1, 1) => self.top_right = tile,
      (-1, 0) => self.left = tile,
      (1, 0) => self.right = tile,
      (-1, -1) => self.bottom_left = tile,
      (0, -1) => self.bottom = tile,
      (1, -1) => self.bottom_right = tile,
      _ => error!(
        "Attempted to add a NeighbourTile with an invalid direction {:?}",
        tile.direction
      ),
    }
  }

  pub fn count_same(&self) -> usize {
    [
      self.top_left,
      self.top,
      self.top_right,
      self.left,
      self.right,
      self.bottom_left,
      self.bottom,
      self.bottom_right,
    ]
    .iter()
    .filter(|&&tile| tile.is_same)
    .count()
  }

  pub fn print(&self, tile: &Tile, neighbour_count: usize) {
    trace!("{:?}", tile);
    trace!("|-------|-------|-------|");
    trace!(
      "| {:<5} | {:<5} | {:<5} |",
      format!("{:?}", self.top_left.terrain)
        .chars()
        .take(5)
        .collect::<String>(),
      format!("{:?}", self.top.terrain).chars().take(5).collect::<String>(),
      format!("{:?}", self.top_right.terrain)
        .chars()
        .take(5)
        .collect::<String>()
    );
    trace!(
      "| {:<5} | {:<5} | {:<5} | => '{:?}' with {} neighbours",
      format!("{:?}", self.left.terrain).chars().take(5).collect::<String>(),
      format!("{:?}", tile.terrain).chars().take(5).collect::<String>(),
      format!("{:?}", self.right.terrain).chars().take(5).collect::<String>(),
      tile.tile_type,
      neighbour_count
    );
    trace!(
      "| {:<5} | {:<5} | {:<5} |",
      format!("{:?}", self.bottom_left.terrain)
        .chars()
        .take(5)
        .collect::<String>(),
      format!("{:?}", self.bottom.terrain).chars().take(5).collect::<String>(),
      format!("{:?}", self.bottom_right.terrain)
        .chars()
        .take(5)
        .collect::<String>()
    );
    trace!("|-------|-------|-------|");
    trace!("");
  }
}
