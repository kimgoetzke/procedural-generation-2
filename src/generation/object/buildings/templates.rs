use crate::coords::Point;
use crate::generation::lib::Direction;
use crate::generation::object::buildings::{BuildingLevel, BuildingTemplate, BuildingType, Level, StructureType, Variants};
use crate::generation::object::lib::ObjectName;
use bevy::platform::collections::HashMap;

#[derive(PartialEq, Eq)]
pub struct BuildingComponentRegistry {
  variants: HashMap<(BuildingType, Level, StructureType), Variants>,
}

impl BuildingComponentRegistry {
  pub fn default() -> Self {
    BuildingComponentRegistry {
      variants: HashMap::new(),
    }
  }

  pub fn new_initialised() -> Self {
    let mut registry = BuildingComponentRegistry::default();

    // Small house - Ground floor doors
    registry.insert_doors_with_3_structures(
      BuildingType::SmallHouse,
      Level::GroundFloor,
      vec![ObjectName::HouseSmallDoorLeft1, ObjectName::HouseSmallDoorLeft2],
      vec![ObjectName::HouseSmallDoorMiddle],
      vec![ObjectName::HouseSmallDoorRight1, ObjectName::HouseSmallDoorRight2],
    );

    // Small house - Ground floor walls
    registry.insert_level_with_3_structures(
      BuildingType::SmallHouse,
      Level::GroundFloor,
      vec![ObjectName::HouseSmallWallLeft],
      vec![ObjectName::HouseSmallWallMiddle1, ObjectName::HouseSmallWallMiddle2],
      vec![ObjectName::HouseSmallWallRight],
    );

    // Small house - Roof
    registry.insert_level_with_3_structures(
      BuildingType::SmallHouse,
      Level::Roof,
      vec![
        ObjectName::HouseSmallRoofLeft1,
        ObjectName::HouseSmallRoofLeft2,
        ObjectName::HouseSmallRoofLeft3,
      ],
      vec![
        ObjectName::HouseSmallRoofMiddle1,
        ObjectName::HouseSmallRoofMiddle2,
        ObjectName::HouseSmallRoofMiddle3,
      ],
      vec![
        ObjectName::HouseSmallRoofRight1,
        ObjectName::HouseSmallRoofRight2,
        ObjectName::HouseSmallRoofRight3,
      ],
    );

    // Medium house - Ground floor doors
    registry.insert_doors_with_3_structures(
      BuildingType::MediumHouse,
      Level::GroundFloor,
      vec![ObjectName::HouseMediumDoorLeft1, ObjectName::HouseMediumDoorLeft2],
      vec![ObjectName::HouseMediumDoorMiddle],
      vec![ObjectName::HouseMediumDoorRight1, ObjectName::HouseMediumDoorRight2],
    );

    // Medium house - Ground floor walls
    registry.insert_level_with_3_structures(
      BuildingType::MediumHouse,
      Level::GroundFloor,
      vec![ObjectName::HouseMediumWallLeft],
      vec![ObjectName::HouseMediumWallMiddle1, ObjectName::HouseMediumWallMiddle2],
      vec![ObjectName::HouseMediumWallRight],
    );

    // Medium house - Roof
    registry.insert_level_with_3_structures(
      BuildingType::MediumHouse,
      Level::Roof,
      vec![
        ObjectName::HouseMediumRoofLeft1,
        ObjectName::HouseMediumRoofLeft2,
        ObjectName::HouseMediumRoofLeft3,
      ],
      vec![
        ObjectName::HouseMediumRoofMiddle1,
        ObjectName::HouseMediumRoofMiddle2,
        ObjectName::HouseMediumRoofMiddle3,
      ],
      vec![
        ObjectName::HouseMediumRoofRight1,
        ObjectName::HouseMediumRoofRight2,
        ObjectName::HouseMediumRoofRight3,
      ],
    );

    registry
  }

  fn insert_level_with_3_structures(
    &mut self,
    building_type: BuildingType,
    level: Level,
    left: Vec<ObjectName>,
    middle: Vec<ObjectName>,
    right: Vec<ObjectName>,
  ) {
    self
      .variants
      .insert((building_type, level, StructureType::Left), Variants::new(left));
    self
      .variants
      .insert((building_type, level, StructureType::Middle), Variants::new(middle));
    self
      .variants
      .insert((building_type, level, StructureType::Right), Variants::new(right));
  }

  fn insert_doors_with_3_structures(
    &mut self,
    building_type: BuildingType,
    level: Level,
    left: Vec<ObjectName>,
    middle: Vec<ObjectName>,
    right: Vec<ObjectName>,
  ) {
    self
      .variants
      .insert((building_type, level, StructureType::LeftDoor), Variants::new(left));
    self
      .variants
      .insert((building_type, level, StructureType::MiddleDoor), Variants::new(middle));
    self
      .variants
      .insert((building_type, level, StructureType::RightDoor), Variants::new(right));
  }

  pub fn get_variants_for(
    &self,
    building_type: &BuildingType,
    level_type: &Level,
    structure_type: &StructureType,
  ) -> Vec<ObjectName> {
    self
      .variants
      .get(&(*building_type, *level_type, *structure_type))
      .unwrap_or(&Variants::empty())
      .variants
      .clone()
  }
}

// TODO: Consider moving this (or even more of this class) to some form of resource file after moving away from RON files
pub fn get_building_templates() -> Vec<BuildingTemplate> {
  vec![
    BuildingTemplate::new(
      "Small House Facing North",
      BuildingType::SmallHouse,
      vec![
        BuildingLevel::standard(Level::Roof),
        BuildingLevel {
          level: Level::GroundFloor,
          structures: vec![StructureType::Left, StructureType::MiddleDoor, StructureType::Right],
        },
      ],
      Point::new_internal_grid(1, 1),
      Direction::Bottom,
    ),
    BuildingTemplate::new(
      "Small House Facing East",
      BuildingType::SmallHouse,
      vec![
        BuildingLevel::standard(Level::Roof),
        BuildingLevel {
          level: Level::GroundFloor,
          structures: vec![StructureType::Left, StructureType::Middle, StructureType::RightDoor],
        },
      ],
      Point::new_internal_grid(2, 1),
      Direction::Right,
    ),
    BuildingTemplate::new(
      "Small House Facing West",
      BuildingType::SmallHouse,
      vec![
        BuildingLevel::standard(Level::Roof),
        BuildingLevel {
          level: Level::GroundFloor,
          structures: vec![StructureType::LeftDoor, StructureType::Middle, StructureType::Right],
        },
      ],
      Point::new_internal_grid(0, 1),
      Direction::Left,
    ),
    BuildingTemplate::new(
      "Medium House Facing North",
      BuildingType::MediumHouse,
      vec![
        BuildingLevel::standard(Level::Roof),
        BuildingLevel {
          level: Level::GroundFloor,
          structures: vec![StructureType::Left, StructureType::MiddleDoor, StructureType::Right],
        },
      ],
      Point::new_internal_grid(1, 1),
      Direction::Bottom,
    ),
    BuildingTemplate::new(
      "Medium House Facing East",
      BuildingType::MediumHouse,
      vec![
        BuildingLevel::standard(Level::Roof),
        BuildingLevel {
          level: Level::GroundFloor,
          structures: vec![StructureType::Left, StructureType::Middle, StructureType::RightDoor],
        },
      ],
      Point::new_internal_grid(2, 1),
      Direction::Right,
    ),
    BuildingTemplate::new(
      "Medium House Facing West",
      BuildingType::MediumHouse,
      vec![
        BuildingLevel::standard(Level::Roof),
        BuildingLevel {
          level: Level::GroundFloor,
          structures: vec![StructureType::LeftDoor, StructureType::Middle, StructureType::Right],
        },
      ],
      Point::new_internal_grid(0, 1),
      Direction::Left,
    ),
  ]
}
