use crate::coords::Point;
use crate::coords::point::CoordType;
use crate::generation::lib::{TerrainType, Tile};
use bevy::log::*;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct NeighbourTile<T: CoordType> {
  pub direction: Point<T>,
  pub terrain: TerrainType,
  pub same: bool,
  pub is_any: bool,
}

impl<T: CoordType> NeighbourTile<T> {
  pub fn default(direction: Point<T>) -> Self {
    Self {
      direction,
      terrain: TerrainType::Any,
      same: false,
      is_any: true,
    }
  }

  pub(crate) fn new(direction: Point<T>, terrain_type: TerrainType, is_same: bool) -> NeighbourTile<T> {
    NeighbourTile {
      direction,
      terrain: terrain_type,
      same: is_same,
      is_any: terrain_type == TerrainType::Any,
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub struct NeighbourTiles<T: CoordType> {
  pub top_left: NeighbourTile<T>,
  pub top: NeighbourTile<T>,
  pub top_right: NeighbourTile<T>,
  pub left: NeighbourTile<T>,
  pub right: NeighbourTile<T>,
  pub bottom_left: NeighbourTile<T>,
  pub bottom: NeighbourTile<T>,
  pub bottom_right: NeighbourTile<T>,
}

#[allow(dead_code)]
impl<T: CoordType> NeighbourTiles<T> {
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

  pub fn all_direction_top_left_same(&self) -> bool {
    self.top_left.same && self.top.same && self.left.same
  }

  pub fn all_direction_top_right_same(&self) -> bool {
    self.top.same && self.top_right.same && self.right.same
  }

  pub fn all_direction_bottom_left_same(&self) -> bool {
    self.left.same && self.bottom.same && self.bottom_left.same
  }

  pub fn all_direction_bottom_right_same(&self) -> bool {
    self.bottom.same && self.bottom_right.same && self.right.same
  }

  pub fn all_direction_top_left_different(&self) -> bool {
    !self.top_left.same && !self.top.same && !self.left.same
  }

  pub fn all_direction_top_right_different(&self) -> bool {
    !self.top.same && !self.top_right.same && !self.right.same
  }

  pub fn all_direction_bottom_left_different(&self) -> bool {
    !self.left.same && !self.bottom.same && !self.bottom_left.same
  }

  pub fn all_direction_bottom_right_different(&self) -> bool {
    !self.bottom.same && !self.bottom_right.same && !self.right.same
  }

  pub fn all_top_same(&self) -> bool {
    self.top_left.same && self.top.same && self.top_right.same
  }

  pub fn all_top_different(&self) -> bool {
    !self.top_left.same && !self.top.same && !self.top_right.same
  }

  pub fn top_same(&self, expected: usize) -> bool {
    [&self.top_left, &self.top, &self.top_right]
      .iter()
      .filter(|&&tile| tile.same)
      .count()
      == expected
  }

  pub fn all_bottom_same(&self) -> bool {
    self.bottom_left.same && self.bottom.same && self.bottom_right.same
  }

  pub fn all_bottom_different(&self) -> bool {
    !self.bottom_left.same && !self.bottom.same && !self.bottom_right.same
  }

  pub fn bottom_same(&self, expected: usize) -> bool {
    [&self.bottom_left, &self.bottom, &self.bottom_right]
      .iter()
      .filter(|tile| tile.same)
      .count()
      == expected
  }

  pub fn all_left_same(&self) -> bool {
    self.top_left.same && self.left.same && self.bottom_left.same
  }

  pub fn all_left_different(&self) -> bool {
    !self.top_left.same && !self.left.same && !self.bottom_left.same
  }

  pub fn left_same(&self, expected: usize) -> bool {
    [&self.top_left, &self.left, &self.bottom_left]
      .iter()
      .filter(|&&tile| tile.same)
      .count()
      == expected
  }

  pub fn all_right_same(&self) -> bool {
    self.top_right.same && self.right.same && self.bottom_right.same
  }

  pub fn all_right_different(&self) -> bool {
    !self.top_right.same && !self.right.same && !self.bottom_right.same
  }

  pub fn right_same(&self, expected: usize) -> bool {
    [&self.top_right, &self.right, &self.bottom_right]
      .iter()
      .filter(|&&tile| tile.same)
      .count()
      == expected
  }

  pub fn all_sides_same(&self) -> bool {
    self.top.same && self.right.same && self.bottom.same && self.left.same
  }

  pub fn put(&mut self, tile: NeighbourTile<T>) {
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
      &self.top_left,
      &self.top,
      &self.top_right,
      &self.left,
      &self.right,
      &self.bottom_left,
      &self.bottom,
      &self.bottom_right,
    ]
    .iter()
    .filter(|&&tile| tile.same)
    .count()
  }

  pub fn log(&self, tile: &Tile, neighbour_count: usize) {
    debug!("{:?}", tile);
    debug!("┌────────┬────────┬────────┐");
    debug!(
      "│ {:<6} │ {:<6} │ {:<6} │",
      format!("{:?}", self.top_left.terrain).chars().take(6).collect::<String>(),
      format!("{:?}", self.top.terrain).chars().take(6).collect::<String>(),
      format!("{:?}", self.top_right.terrain).chars().take(6).collect::<String>()
    );
    debug!(
      "│ {:<6} │ {:<6} │ {:<6} │ => '{:?}' with {} neighbours",
      format!("{:?}", self.left.terrain).chars().take(6).collect::<String>(),
      format!("{:?}", tile.terrain).chars().take(6).collect::<String>(),
      format!("{:?}", self.right.terrain).chars().take(6).collect::<String>(),
      tile.tile_type,
      neighbour_count
    );
    debug!(
      "│ {:<6} │ {:<6} │ {:<6} │",
      format!("{:?}", self.bottom_left.terrain).chars().take(6).collect::<String>(),
      format!("{:?}", self.bottom.terrain).chars().take(6).collect::<String>(),
      format!("{:?}", self.bottom_right.terrain).chars().take(6).collect::<String>()
    );
    debug!("└────────┴────────┴────────┘");
    debug!("");
  }
}
