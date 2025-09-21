use crate::generation::object::buildings::templates::{BuildingType, Level, StructureType};
use crate::generation::object::lib::ObjectName;
use bevy::platform::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Variants {
  variants: Vec<ObjectName>,
}

impl Variants {
  pub fn empty() -> Self {
    Self { variants: vec![] }
  }

  pub fn new(variants: Vec<ObjectName>) -> Self {
    Self { variants }
  }
}

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
    registry.insert_doors(
      BuildingType::SmallHouse,
      Level::GroundFloor,
      vec![ObjectName::HouseSmallDoorLeft],
      None,
      vec![ObjectName::HouseSmallDoorRight],
    );

    // Small house - Ground floor walls
    registry.insert_level(
      BuildingType::SmallHouse,
      Level::GroundFloor,
      vec![ObjectName::HouseSmallWallLeft1, ObjectName::HouseSmallWallLeft2],
      None,
      vec![ObjectName::HouseSmallWallRight1, ObjectName::HouseSmallWallRight2],
    );

    // Small house - Roof
    registry.insert_level(
      BuildingType::SmallHouse,
      Level::Roof,
      vec![
        ObjectName::HouseSmallRoofLeft1,
        ObjectName::HouseSmallRoofLeft2,
        ObjectName::HouseSmallRoofLeft3,
      ],
      None,
      vec![
        ObjectName::HouseSmallRoofRight1,
        ObjectName::HouseSmallRoofRight2,
        ObjectName::HouseSmallRoofRight3,
      ],
    );

    // Medium house - Ground floor doors
    registry.insert_doors(
      BuildingType::MediumHouse,
      Level::GroundFloor,
      vec![ObjectName::HouseMediumDoorLeft1, ObjectName::HouseMediumDoorLeft2],
      Some(vec![ObjectName::HouseMediumDoorMiddle]),
      vec![ObjectName::HouseMediumDoorRight1, ObjectName::HouseMediumDoorRight2],
    );

    // Medium house - Ground floor walls
    registry.insert_level(
      BuildingType::MediumHouse,
      Level::GroundFloor,
      vec![ObjectName::HouseMediumWallLeft],
      Some(vec![ObjectName::HouseMediumWallMiddle1, ObjectName::HouseMediumWallMiddle2]),
      vec![ObjectName::HouseMediumWallRight],
    );

    // Medium house - Roof
    registry.insert_level(
      BuildingType::MediumHouse,
      Level::Roof,
      vec![
        ObjectName::HouseMediumRoofLeft1,
        ObjectName::HouseMediumRoofLeft2,
        ObjectName::HouseMediumRoofLeft3,
      ],
      Some(vec![
        ObjectName::HouseMediumRoofMiddle1,
        ObjectName::HouseMediumRoofMiddle2,
        ObjectName::HouseMediumRoofMiddle3,
      ]),
      vec![
        ObjectName::HouseMediumRoofRight1,
        ObjectName::HouseMediumRoofRight2,
        ObjectName::HouseMediumRoofRight3,
      ],
    );

    // Large house - Ground floor doors
    registry.insert_doors(
      BuildingType::LargeHouse,
      Level::GroundFloor,
      vec![ObjectName::HouseLargeDoorLeft1, ObjectName::HouseLargeDoorLeft2],
      Some(vec![ObjectName::HouseLargeDoorMiddle]),
      vec![ObjectName::HouseLargeDoorRight1, ObjectName::HouseLargeDoorRight2],
    );

    // Large house - Ground floor walls
    registry.insert_level(
      BuildingType::LargeHouse,
      Level::GroundFloor,
      vec![ObjectName::HouseLargeWallLeft],
      Some(vec![ObjectName::HouseLargeWallMiddle1, ObjectName::HouseLargeWallMiddle2]),
      vec![ObjectName::HouseLargeWallRight],
    );

    // Large house - Roof
    registry.insert_level(
      BuildingType::LargeHouse,
      Level::Roof,
      vec![
        ObjectName::HouseLargeRoofLeft1,
        ObjectName::HouseLargeRoofLeft2,
        ObjectName::HouseLargeRoofLeft3,
      ],
      Some(vec![
        ObjectName::HouseLargeRoofMiddle1,
        ObjectName::HouseLargeRoofMiddle2,
        ObjectName::HouseLargeRoofMiddle3,
      ]),
      vec![
        ObjectName::HouseLargeRoofRight1,
        ObjectName::HouseLargeRoofRight2,
        ObjectName::HouseLargeRoofRight3,
      ],
    );

    registry
  }

  fn insert_level(
    &mut self,
    building_type: BuildingType,
    level: Level,
    left: Vec<ObjectName>,
    middle: Option<Vec<ObjectName>>,
    right: Vec<ObjectName>,
  ) {
    self
      .variants
      .insert((building_type, level, StructureType::Left), Variants::new(left));
    if middle.is_some() {
      self
        .variants
        .insert((building_type, level, StructureType::Middle), Variants::new(middle.unwrap()));
    }
    self
      .variants
      .insert((building_type, level, StructureType::Right), Variants::new(right));
  }

  fn insert_doors(
    &mut self,
    building_type: BuildingType,
    level: Level,
    left: Vec<ObjectName>,
    middle: Option<Vec<ObjectName>>,
    right: Vec<ObjectName>,
  ) {
    self
      .variants
      .insert((building_type, level, StructureType::LeftDoor), Variants::new(left));
    if middle.is_some() {
      self.variants.insert(
        (building_type, level, StructureType::MiddleDoor),
        Variants::new(middle.unwrap()),
      );
    }
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn insert_level_with_3_structures_inserts_all_structures_correctly() {
    let mut registry = BuildingComponentRegistry::default();
    let building_type = BuildingType::MediumHouse;
    let level = Level::GroundFloor;
    let left = vec![ObjectName::HouseMediumWallLeft];
    let middle = vec![ObjectName::HouseMediumWallMiddle1];
    let right = vec![ObjectName::HouseMediumWallRight];

    registry.insert_level(building_type, level, left.clone(), Some(middle.clone()), right.clone());

    assert_eq!(registry.get_variants_for(&building_type, &level, &StructureType::Left), left);
    assert_eq!(
      registry.get_variants_for(&building_type, &level, &StructureType::Middle),
      middle
    );
    assert_eq!(
      registry.get_variants_for(&building_type, &level, &StructureType::Right),
      right
    );
  }

  #[test]
  fn insert_level_with_3_structures_overwrites_existing_entries() {
    let mut registry = BuildingComponentRegistry::default();
    let building_type = BuildingType::MediumHouse;
    let level = Level::GroundFloor;
    let initial_left = vec![ObjectName::HouseMediumWallLeft];
    let new_left = vec![ObjectName::HouseMediumDoorLeft1];

    registry.insert_level(building_type, level, initial_left.clone(), None, vec![]);
    registry.insert_level(building_type, level, new_left.clone(), None, vec![]);

    assert_eq!(
      registry.get_variants_for(&building_type, &level, &StructureType::Left),
      new_left
    );
  }

  #[test]
  fn insert_level_with_3_structures_handles_empty_variants() {
    let mut registry = BuildingComponentRegistry::default();
    let building_type = BuildingType::MediumHouse;
    let level = Level::GroundFloor;

    registry.insert_level(building_type, level, vec![], None, vec![]);

    assert!(
      registry
        .get_variants_for(&building_type, &level, &StructureType::Left)
        .is_empty()
    );
    assert!(
      registry
        .get_variants_for(&building_type, &level, &StructureType::Middle)
        .is_empty()
    );
    assert!(
      registry
        .get_variants_for(&building_type, &level, &StructureType::Right)
        .is_empty()
    );
  }

  #[test]
  fn insert_doors_with_3_structures_inserts_all_doors_correctly() {
    let mut registry = BuildingComponentRegistry::default();
    let building_type = BuildingType::MediumHouse;
    let level = Level::GroundFloor;
    let left = vec![ObjectName::HouseMediumDoorLeft1];
    let middle = vec![ObjectName::HouseMediumDoorMiddle];
    let right = vec![ObjectName::HouseMediumDoorRight1];

    registry.insert_doors(building_type, level, left.clone(), Some(middle.clone()), right.clone());

    assert_eq!(
      registry.get_variants_for(&building_type, &level, &StructureType::LeftDoor),
      left
    );
    assert_eq!(
      registry.get_variants_for(&building_type, &level, &StructureType::MiddleDoor),
      middle
    );
    assert_eq!(
      registry.get_variants_for(&building_type, &level, &StructureType::RightDoor),
      right
    );
  }

  #[test]
  fn insert_doors_with_3_structures_overwrites_existing_door_entries() {
    let mut registry = BuildingComponentRegistry::default();
    let building_type = BuildingType::MediumHouse;
    let level = Level::GroundFloor;
    let initial_left = vec![ObjectName::HouseMediumDoorLeft1];
    let new_left = vec![ObjectName::HouseMediumDoorLeft2];

    registry.insert_doors(building_type, level, initial_left.clone(), None, vec![]);
    registry.insert_doors(building_type, level, new_left.clone(), None, vec![]);

    assert_eq!(
      registry.get_variants_for(&building_type, &level, &StructureType::LeftDoor),
      new_left
    );
  }

  #[test]
  fn insert_doors_with_3_structures_handles_empty_door_variants() {
    let mut registry = BuildingComponentRegistry::default();
    let building_type = BuildingType::MediumHouse;
    let level = Level::GroundFloor;

    registry.insert_doors(building_type, level, vec![], None, vec![]);

    assert!(
      registry
        .get_variants_for(&building_type, &level, &StructureType::LeftDoor)
        .is_empty()
    );
    assert!(
      registry
        .get_variants_for(&building_type, &level, &StructureType::MiddleDoor)
        .is_empty()
    );
    assert!(
      registry
        .get_variants_for(&building_type, &level, &StructureType::RightDoor)
        .is_empty()
    );
  }
}
