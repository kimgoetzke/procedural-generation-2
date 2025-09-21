use crate::constants::CHUNK_SIZE;
use crate::coords::Point;
use crate::coords::point::InternalGrid;
use crate::generation::lib::Direction;
use crate::generation::object::buildings::registry::BuildingComponentRegistry;
use crate::generation::object::lib::ObjectName;
use bevy::platform::collections::HashSet;
use rand::Rng;
use rand::prelude::StdRng;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuildingType {
  SmallHouse,
  MediumHouse,
  LargeHouse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Level {
  GroundFloor,
  Roof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StructureType {
  Left,
  Middle,
  Right,
  LeftDoor,
  MiddleDoor,
  RightDoor,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BuildingLevel {
  pub level: Level,
  pub structures: Vec<StructureType>,
}

impl BuildingLevel {
  pub fn left_right_structure(level: Level) -> Self {
    Self {
      level,
      structures: vec![StructureType::Left, StructureType::Right],
    }
  }

  pub fn left_middle_right_structure(level: Level) -> Self {
    Self {
      level,
      structures: vec![StructureType::Left, StructureType::Middle, StructureType::Right],
    }
  }
}

#[derive(Debug, Clone)]
pub(crate) struct BuildingTemplate {
  pub(crate) name: String,
  building_type: BuildingType,
  pub(crate) width: i32,
  pub(crate) height: i32,
  layout: Vec<BuildingLevel>,
  /// The position of the door tile relative to the building's top-left corner in internal grid coordinates.  Remember
  /// that `x` is the column number and `y` is the row number of the door's position in the `tiles` 2D array.
  relative_door_ig: Point<InternalGrid>,
  connection_direction: Direction,
}

impl BuildingTemplate {
  pub fn new(
    name: &str,
    building_type: BuildingType,
    layout: Vec<BuildingLevel>,
    relative_door_ig: Point<InternalGrid>,
    connection_direction: Direction,
  ) -> Self {
    let height = layout.len() as i32;
    let width = if height > 0 { layout[0].structures.len() as i32 } else { 0 };

    Self {
      name: name.to_string(),
      building_type,
      width,
      height,
      layout,
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

  pub(crate) fn generate_tiles(&self, registry: &BuildingComponentRegistry, rng: &mut StdRng) -> Vec<Vec<ObjectName>> {
    let mut tiles = vec![vec![ObjectName::Empty; self.width as usize]; self.height as usize];
    for (y, level) in self.layout.iter().enumerate() {
      for (x, structure_type) in level.structures.iter().enumerate() {
        let variants = registry.get_variants_for(&self.building_type, &level.level, structure_type);
        if variants.is_empty() {
          panic!(
            "No variants found for building type [{:?}], level [{:?}], component type [{:?}] in building template [{}] - this indicates a configuration error in the BuildingComponentRegistry",
            self.building_type, level.level, structure_type, self.name
          );
        }
        let selected_variant = rng.random_range(0..variants.len());
        tiles[y][x] = variants[selected_variant];
      }
    }

    tiles
  }
}

pub(crate) fn get_building_templates() -> Vec<BuildingTemplate> {
  vec![
    BuildingTemplate::new(
      "Small House Facing East",
      BuildingType::SmallHouse,
      vec![
        BuildingLevel::left_right_structure(Level::Roof),
        BuildingLevel {
          level: Level::GroundFloor,
          structures: vec![StructureType::Left, StructureType::RightDoor],
        },
      ],
      Point::new_internal_grid(1, 1), // Reminder: x is column, y is row
      Direction::Right,
    ),
    BuildingTemplate::new(
      "Small House Facing West",
      BuildingType::SmallHouse,
      vec![
        BuildingLevel::left_right_structure(Level::Roof),
        BuildingLevel {
          level: Level::GroundFloor,
          structures: vec![StructureType::LeftDoor, StructureType::Right],
        },
      ],
      Point::new_internal_grid(0, 1), // Reminder: x is column, y is row
      Direction::Left,
    ),
    BuildingTemplate::new(
      "Medium House Facing North",
      BuildingType::MediumHouse,
      vec![
        BuildingLevel::left_middle_right_structure(Level::Roof),
        BuildingLevel {
          level: Level::GroundFloor,
          structures: vec![StructureType::Left, StructureType::MiddleDoor, StructureType::Right],
        },
      ],
      Point::new_internal_grid(1, 1), // Reminder: x is column, y is row
      Direction::Bottom,
    ),
    BuildingTemplate::new(
      "Medium House Facing East",
      BuildingType::MediumHouse,
      vec![
        BuildingLevel::left_middle_right_structure(Level::Roof),
        BuildingLevel {
          level: Level::GroundFloor,
          structures: vec![StructureType::Left, StructureType::Middle, StructureType::RightDoor],
        },
      ],
      Point::new_internal_grid(2, 1), // Reminder: x is column, y is row
      Direction::Right,
    ),
    BuildingTemplate::new(
      "Medium House Facing West",
      BuildingType::MediumHouse,
      vec![
        BuildingLevel::left_middle_right_structure(Level::Roof),
        BuildingLevel {
          level: Level::GroundFloor,
          structures: vec![StructureType::LeftDoor, StructureType::Middle, StructureType::Right],
        },
      ],
      Point::new_internal_grid(0, 1), // Reminder: x is column, y is row
      Direction::Left,
    ),
    BuildingTemplate::new(
      "Large House Facing North",
      BuildingType::LargeHouse,
      vec![
        BuildingLevel::left_middle_right_structure(Level::Roof),
        BuildingLevel {
          level: Level::GroundFloor,
          structures: vec![StructureType::Left, StructureType::MiddleDoor, StructureType::Right],
        },
      ],
      Point::new_internal_grid(1, 1), // Reminder: x is column, y is row
      Direction::Bottom,
    ),
    BuildingTemplate::new(
      "Large House Facing East",
      BuildingType::LargeHouse,
      vec![
        BuildingLevel::left_middle_right_structure(Level::Roof),
        BuildingLevel {
          level: Level::GroundFloor,
          structures: vec![StructureType::Left, StructureType::Middle, StructureType::RightDoor],
        },
      ],
      Point::new_internal_grid(2, 1), // Reminder: x is column, y is row
      Direction::Right,
    ),
    BuildingTemplate::new(
      "Large House Facing West",
      BuildingType::LargeHouse,
      vec![
        BuildingLevel::left_middle_right_structure(Level::Roof),
        BuildingLevel {
          level: Level::GroundFloor,
          structures: vec![StructureType::LeftDoor, StructureType::Middle, StructureType::Right],
        },
      ],
      Point::new_internal_grid(0, 1), // Reminder: x is column, y is row
      Direction::Left,
    ),
  ]
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::coords::Point;
  use crate::generation::lib::Direction;
  use bevy::platform::collections::HashSet;
  use rand::SeedableRng;
  use rand::rngs::StdRng;

  #[test]
  fn calculate_origin_ig_from_connection_point_correctly_calculates_origin() {
    let template = BuildingTemplate::new(
      "Test Building",
      BuildingType::MediumHouse,
      vec![BuildingLevel::left_middle_right_structure(Level::GroundFloor)],
      Point::new_internal_grid(1, 1),
      Direction::Bottom,
    );
    let connection_ig = Point::new_internal_grid(5, 5);
    let origin_ig = template.calculate_origin_ig_from_connection_point(connection_ig);
    assert_eq!(origin_ig, Point::new_internal_grid(4, 3)); // Because building is 1x1 and door is touching connection point
  }

  #[test]
  fn calculate_origin_ig_from_absolute_door_correctly_calculates_origin() {
    let template = BuildingTemplate::new(
      "Test Building",
      BuildingType::MediumHouse,
      vec![BuildingLevel::left_middle_right_structure(Level::GroundFloor)],
      Point::new_internal_grid(1, 1),
      Direction::Bottom,
    );
    let absolute_door = Point::new_internal_grid(6, 6);
    let origin = template.calculate_origin_ig_from_absolute_door(absolute_door);
    assert_eq!(origin, Point::new_internal_grid(5, 5)); // Because building is 1x1 and door is assumed at (6, 6)
  }

  #[test]
  fn calculate_absolute_door_ig_correctly_calculates_door_position() {
    let template = BuildingTemplate::new(
      "Test Building",
      BuildingType::MediumHouse,
      vec![BuildingLevel::left_middle_right_structure(Level::GroundFloor)],
      Point::new_internal_grid(1, 1),
      Direction::Bottom,
    );
    let connection_point = Point::new_internal_grid(5, 5);
    let door_position = template.calculate_absolute_door_ig(connection_point);
    assert_eq!(door_position, Point::new_internal_grid(5, 4));
  }

  #[test]
  fn is_placeable_at_path_returns_true_for_valid_placement() {
    let template = BuildingTemplate::new(
      "Test Building",
      BuildingType::MediumHouse,
      vec![BuildingLevel::left_middle_right_structure(Level::GroundFloor)],
      Point::new_internal_grid(0, 1),
      Direction::Top,
    );
    let connection_point = Point::new_internal_grid(0, 0);
    let mut available_space = HashSet::new();
    for x in 0..3 {
      // 3 because the building is 3 tiles wide and the door is at (0, 1)
      for y in 0..1 {
        // 1 because the building is 1 tile tall
        available_space.insert(Point::new_internal_grid(x, y));
      }
    }
    assert!(template.is_placeable_at_path(connection_point, &available_space));
  }

  #[test]
  fn is_placeable_at_path_returns_false_for_out_of_bounds() {
    let template = BuildingTemplate::new(
      "Test Building",
      BuildingType::MediumHouse,
      vec![BuildingLevel::left_middle_right_structure(Level::GroundFloor)],
      Point::new_internal_grid(1, 1),
      Direction::Bottom,
    );
    let connection_point = Point::new_internal_grid(-1, -1);
    let available_space = HashSet::new();
    assert!(!template.is_placeable_at_path(connection_point, &available_space));
  }

  #[test]
  fn is_placeable_at_path_returns_false_if_space_is_unavailable() {
    let template = BuildingTemplate::new(
      "Test Building",
      BuildingType::MediumHouse,
      vec![BuildingLevel::left_middle_right_structure(Level::GroundFloor)],
      Point::new_internal_grid(1, 1),
      Direction::Bottom,
    );
    let connection_point = Point::new_internal_grid(5, 5);
    let available_space = HashSet::new();
    assert!(!template.is_placeable_at_path(connection_point, &available_space));
  }

  #[test]
  fn generate_tiles_creates_correct_tile_layout() {
    let template = BuildingTemplate::new(
      "Test Building",
      BuildingType::MediumHouse,
      vec![
        BuildingLevel::left_middle_right_structure(Level::GroundFloor),
        BuildingLevel::left_middle_right_structure(Level::Roof),
      ],
      Point::new_internal_grid(1, 1),
      Direction::Bottom,
    );
    let mut rng = StdRng::seed_from_u64(42);
    let registry = BuildingComponentRegistry::new_initialised();
    let tiles = template.generate_tiles(&registry, &mut rng);
    assert_eq!(tiles.len(), template.height as usize);
    assert_eq!(tiles[0].len(), template.width as usize);
    assert_eq!(tiles[0][0], ObjectName::HouseMediumWallLeft);
    assert_eq!(tiles[0][1], ObjectName::HouseMediumWallMiddle2);
    assert_eq!(tiles[0][2], ObjectName::HouseMediumWallRight);
    assert_eq!(tiles[1][0], ObjectName::HouseMediumRoofLeft2);
    assert_eq!(tiles[1][1], ObjectName::HouseMediumRoofMiddle3);
    assert_eq!(tiles[1][2], ObjectName::HouseMediumRoofRight2);
  }
}
