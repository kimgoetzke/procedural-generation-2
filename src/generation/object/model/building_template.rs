use crate::constants::CHUNK_SIZE;
use crate::coordinates::point::InternalGrid;
use crate::coordinates::{Direction, Point};
use crate::generation::object::model::ObjectName;
use bevy::platform::collections::HashSet;
use rand::RngExt;
use rand::prelude::StdRng;

/// A resolved building layout ready for placement.
#[derive(Debug, Clone)]
pub(crate) struct BuildingTemplate {
  pub(crate) id: String,
  pub(crate) width: i32,
  pub(crate) height: i32,
  tile_variants: Vec<Vec<Vec<ObjectName>>>,
  /// The position of the door tile relative to the building's top-left corner in internal grid coordinates.  Remember
  /// that `x` is the column number and `y` is the row number of the door's position in the `tiles` 2D array.
  relative_door_ig: Point<InternalGrid>,
  connection_direction: Direction,
}

impl BuildingTemplate {
  pub(crate) fn new(
    id: String,
    width: i32,
    height: i32,
    tile_variants: Vec<Vec<Vec<ObjectName>>>,
    relative_door_ig: Point<InternalGrid>,
    connection_direction: Direction,
  ) -> Self {
    Self {
      id,
      width,
      height,
      tile_variants,
      relative_door_ig,
      connection_direction,
    }
  }

  /// Calculates where the building's top-left corner should be placed given a connection point which is one tile away
  /// from connection point in the opposite direction.
  pub(crate) fn calculate_origin_ig_from_connection_point(&self, connection_ig: Point<InternalGrid>) -> Point<InternalGrid> {
    let absolute_door_ig = self.calculate_absolute_door_ig(connection_ig);

    Point::new_internal_grid(
      absolute_door_ig.x - self.relative_door_ig.x,
      absolute_door_ig.y - self.relative_door_ig.y,
    )
  }

  /// Calculates where the building's top-left corner should be placed given the absolute position of the door tile in
  /// internal grid coordinates.
  pub(crate) fn calculate_origin_ig_from_absolute_door(&self, absolute_door_ig: Point<InternalGrid>) -> Point<InternalGrid> {
    Point::new_internal_grid(
      absolute_door_ig.x - self.relative_door_ig.x,
      absolute_door_ig.y - self.relative_door_ig.y,
    )
  }

  /// Calculates the absolute position of the door tile in internal grid coordinates based on the connection point and
  /// the connection direction.
  pub(crate) fn calculate_absolute_door_ig(&self, path_ig: Point<InternalGrid>) -> Point<InternalGrid> {
    match self.connection_direction {
      Direction::Top => Point::new_internal_grid(path_ig.x, path_ig.y + 1),
      Direction::Bottom => Point::new_internal_grid(path_ig.x, path_ig.y - 1),
      Direction::Left => Point::new_internal_grid(path_ig.x + 1, path_ig.y),
      Direction::Right => Point::new_internal_grid(path_ig.x - 1, path_ig.y),
      _ => panic!("Invalid connection direction for building template"),
    }
  }

  pub(crate) fn is_placeable_at_path(
    &self,
    path_ig: Point<InternalGrid>,
    available_space: &HashSet<Point<InternalGrid>>,
  ) -> bool {
    let building_origin_ig = self.calculate_origin_ig_from_connection_point(path_ig);

    // Don't allow buildings to be placed out of bounds
    if building_origin_ig.x < 0
      || building_origin_ig.y < 0
      || building_origin_ig.x + self.width > CHUNK_SIZE
      || building_origin_ig.y + self.height > CHUNK_SIZE
    {
      return false;
    }

    // Make sure all tiles the building will occupy are available
    for y in 0..self.height {
      for x in 0..self.width {
        let tile_ig = Point::new_internal_grid(building_origin_ig.x + x, building_origin_ig.y + y);
        if !available_space.contains(&tile_ig) {
          return false;
        }
      }
    }

    // Ensure that the door is next to the connection point and facing it
    let door_ig = Point::new_internal_grid(
      building_origin_ig.x + self.relative_door_ig.x,
      building_origin_ig.y + self.relative_door_ig.y,
    );
    let connection_point_direction: Point<InternalGrid> = self.connection_direction.to_opposite().to_point();
    let expected_door_ig = Point::new_internal_grid(
      path_ig.x + connection_point_direction.x,
      path_ig.y + connection_point_direction.y,
    );
    door_ig == expected_door_ig
  }

  pub(crate) fn generate_tiles(&self, rng: &mut StdRng) -> Vec<Vec<ObjectName>> {
    self
      .tile_variants
      .iter()
      .map(|row| {
        row
          .iter()
          .map(|variants| variants[rng.random_range(0..variants.len())])
          .collect()
      })
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::generation::object::settlements::test_settlement_resources;
  use rand::SeedableRng;

  fn template(id: &str) -> BuildingTemplate {
    test_settlement_resources()
      .building_templates()
      .iter()
      .find(|template| template.id == id)
      .unwrap()
      .clone()
  }

  #[test]
  fn calculate_origin_ig_from_connection_point_correctly_calculates_origin() {
    let origin =
      template("medium_house_facing_north").calculate_origin_ig_from_connection_point(Point::new_internal_grid(5, 5));

    assert_eq!(origin, Point::new_internal_grid(4, 3)); // Because building is 1x1 and door is touching connection point
  }

  #[test]
  fn calculate_origin_ig_from_absolute_door_correctly_calculates_origin() {
    let origin =
      template("medium_house_facing_north").calculate_origin_ig_from_absolute_door(Point::new_internal_grid(6, 6));

    assert_eq!(origin, Point::new_internal_grid(5, 5)); // Because building is 1x1 and door is assumed at (6, 6)
  }

  #[test]
  fn calculate_absolute_door_ig_correctly_calculates_door_position() {
    let door = template("medium_house_facing_north").calculate_absolute_door_ig(Point::new_internal_grid(5, 5));

    assert_eq!(door, Point::new_internal_grid(5, 4));
  }

  #[test]
  fn is_placeable_at_path_returns_true_for_valid_placement() {
    let template = template("medium_house_facing_north");
    let mut available_space = HashSet::new();
    for y in 3..5 {
      for x in 4..7 {
        available_space.insert(Point::new_internal_grid(x, y));
      }
    }

    assert!(template.is_placeable_at_path(Point::new_internal_grid(5, 5), &available_space));
  }

  #[test]
  fn is_placeable_at_path_returns_false_for_out_of_bounds() {
    assert!(!template("medium_house_facing_north").is_placeable_at_path(Point::new_internal_grid(-1, -1), &HashSet::new()));
  }

  #[test]
  fn is_placeable_at_path_returns_false_if_space_is_unavailable() {
    assert!(!template("medium_house_facing_north").is_placeable_at_path(Point::new_internal_grid(5, 5), &HashSet::new()));
  }

  #[test]
  fn generate_tiles_creates_configured_layout() {
    let tiles = template("medium_house_facing_north").generate_tiles(&mut StdRng::seed_from_u64(42));

    assert_eq!(tiles.len(), 2);
    assert_eq!(tiles[0].len(), 3);
    assert!(matches!(
      tiles[0][0],
      ObjectName::HouseMediumRoofLeft1 | ObjectName::HouseMediumRoofLeft2 | ObjectName::HouseMediumRoofLeft3
    ));
    assert!(matches!(
      tiles[1][1],
      ObjectName::HouseMediumDoorMiddle1 | ObjectName::HouseMediumDoorMiddle2
    ));
  }
}
