use bevy::log::*;
use bevy::reflect::Reflect;
use strum::EnumIter;

#[derive(serde::Deserialize, PartialEq, Debug, Clone, Copy, Reflect, Eq, Hash, EnumIter)]
pub enum ObjectName {
  Empty,
  SandStone1,
  SandStone2,
  SandStone3,
  SandStone4,
  SandStone5,
  SandStone6,
  SandGrassPatch1,
  SandGrassPatch2,
  SandPattern1,
  SandPattern2,
  SandPattern3,
  SandPattern4,
  SandPattern5,
  SandPathLeft,
  SandPathRight,
  SandPathTop,
  SandPathBottom,
  SandPathCross,
  SandPathHorizontal,
  SandPathVertical,
  SandStoneTopFill1,
  SandStoneTopFill2,
  SandStoneTopRightFill,
  SandStoneTopLeftFill,
  SandStoneRightFill,
  SandStoneLeftFill,
  SandStoneBottomRightFill,
  SandStoneBottomLeftFill,
  GrassRubbleLeft,
  GrassRubbleRight,
  GrassRubbleTop,
  GrassRubbleBottom,
  GrassRubbleCross,
  GrassRubbleHorizontal,
  GrassRubbleVertical,
  GrassRubbleVerticalForestTop,
  GrassRubbleVerticalForestBottom,
  GrassRubbleHorizontalForestRight,
  GrassRubbleHorizontalForestLeft,
  GrassBush1,
  GrassBush2,
  GrassBush3,
  GrassBush4,
  GrassFlower1,
  GrassFlower2,
  GrassFlower3,
  ForestRuinLeft,
  ForestRuinRight,
  ForestRuinTop,
  ForestRuinBottom,
  ForestRuinCross,
  ForestRuinHorizontal,
  ForestRuinVertical,
  ForestRuinVerticalGrassTop,
  ForestRuinVerticalGrassBottom,
  ForestRuinHorizontalGrassRight,
  ForestRuinHorizontalGrassLeft,
  ForestTree1,
  ForestTree2,
  ForestTree3,
  ForestTree4,
  ForestTree5,
  ForestBush1,
  ForestBush2,
  ForestBush3,
  ForestBush4,
  Land3Grass1,
  Land3Grass2,
  PathRight,
  PathHorizontal,
  PathCross,
  PathVertical,
  PathBottom,
  PathTop,
  PathLeft,
  PathTopRight,
  PathTopLeft,
  PathBottomRight,
  PathBottomLeft,
  PathTopHorizontal,
  PathBottomHorizontal,
  PathLeftVertical,
  PathRightVertical,
  PathUndefined,
  HouseSmallRoofLeft1,
  HouseSmallRoofRight1,
  HouseSmallWallLeft1,
  HouseSmallWallRight1,
  HouseSmallRoofLeft2,
  HouseSmallRoofRight2,
  HouseSmallDoorLeft,
  HouseSmallDoorRight,
  HouseSmallRoofLeft3,
  HouseSmallRoofRight3,
  HouseSmallWallLeft2,
  HouseSmallWallRight2,
  HouseMediumRoofLeft1,
  HouseMediumRoofMiddle1,
  HouseMediumRoofRight1,
  HouseMediumWallLeft,
  HouseMediumDoorMiddle1,
  HouseMediumWallRight,
  HouseMediumRoofLeft2,
  HouseMediumRoofMiddle2,
  HouseMediumRoofRight2,
  HouseMediumDoorLeft1,
  HouseMediumWallMiddle1,
  HouseMediumDoorRight1,
  HouseMediumRoofLeft3,
  HouseMediumRoofMiddle3,
  HouseMediumRoofRight3,
  HouseMediumDoorLeft2,
  HouseMediumWallMiddle2,
  HouseMediumDoorRight2,
  HouseMediumDoorMiddle2,
  HouseLargeRoofLeft1,
  HouseLargeRoofMiddle1,
  HouseLargeRoofRight1,
  HouseLargeWallLeft,
  HouseLargeDoorMiddle1,
  HouseLargeWallRight,
  HouseLargeRoofLeft2,
  HouseLargeRoofMiddle2,
  HouseLargeRoofRight2,
  HouseLargeDoorLeft1,
  HouseLargeWallMiddle1,
  HouseLargeDoorRight1,
  HouseLargeRoofLeft3,
  HouseLargeRoofMiddle3,
  HouseLargeRoofRight3,
  HouseLargeDoorLeft2,
  HouseLargeWallMiddle2,
  HouseLargeDoorRight2,
  HouseLargeDoorMiddle2,
}

impl ObjectName {
  pub fn is_multi_tile(&self) -> bool {
    matches!(
      self,
      ObjectName::ForestTree1
        | ObjectName::ForestTree2
        | ObjectName::ForestTree3
        | ObjectName::ForestTree4
        | ObjectName::ForestTree5
    )
  }

  pub fn is_path(&self) -> bool {
    matches!(
      self,
      ObjectName::PathUndefined
        | ObjectName::PathRight
        | ObjectName::PathHorizontal
        | ObjectName::PathCross
        | ObjectName::PathVertical
        | ObjectName::PathBottom
        | ObjectName::PathTop
        | ObjectName::PathLeft
        | ObjectName::PathTopRight
        | ObjectName::PathTopLeft
        | ObjectName::PathBottomRight
        | ObjectName::PathBottomLeft
        | ObjectName::PathTopHorizontal
        | ObjectName::PathBottomHorizontal
        | ObjectName::PathLeftVertical
        | ObjectName::PathRightVertical
    )
  }

  pub fn is_building(&self) -> bool {
    matches!(
      self,
      ObjectName::HouseSmallRoofLeft1
        | ObjectName::HouseSmallRoofRight1
        | ObjectName::HouseSmallWallLeft1
        | ObjectName::HouseSmallWallRight1
        | ObjectName::HouseSmallRoofLeft2
        | ObjectName::HouseSmallRoofRight2
        | ObjectName::HouseSmallDoorLeft
        | ObjectName::HouseSmallDoorRight
        | ObjectName::HouseSmallRoofLeft3
        | ObjectName::HouseSmallRoofRight3
        | ObjectName::HouseSmallWallLeft2
        | ObjectName::HouseSmallWallRight2
        | ObjectName::HouseMediumRoofLeft1
        | ObjectName::HouseMediumRoofMiddle1
        | ObjectName::HouseMediumRoofRight1
        | ObjectName::HouseMediumWallLeft
        | ObjectName::HouseMediumDoorMiddle1
        | ObjectName::HouseMediumWallRight
        | ObjectName::HouseMediumRoofLeft2
        | ObjectName::HouseMediumRoofMiddle2
        | ObjectName::HouseMediumRoofRight2
        | ObjectName::HouseMediumDoorLeft1
        | ObjectName::HouseMediumWallMiddle1
        | ObjectName::HouseMediumDoorRight1
        | ObjectName::HouseMediumRoofLeft3
        | ObjectName::HouseMediumRoofMiddle3
        | ObjectName::HouseMediumRoofRight3
        | ObjectName::HouseMediumDoorLeft2
        | ObjectName::HouseMediumWallMiddle2
        | ObjectName::HouseMediumDoorRight2
        | ObjectName::HouseMediumDoorMiddle2
        | ObjectName::HouseLargeRoofLeft1
        | ObjectName::HouseLargeRoofMiddle1
        | ObjectName::HouseLargeRoofRight1
        | ObjectName::HouseLargeWallLeft
        | ObjectName::HouseLargeDoorMiddle1
        | ObjectName::HouseLargeWallRight
        | ObjectName::HouseLargeRoofLeft2
        | ObjectName::HouseLargeRoofMiddle2
        | ObjectName::HouseLargeRoofRight2
        | ObjectName::HouseLargeDoorLeft1
        | ObjectName::HouseLargeWallMiddle1
        | ObjectName::HouseLargeDoorRight1
        | ObjectName::HouseLargeRoofLeft3
        | ObjectName::HouseLargeRoofMiddle3
        | ObjectName::HouseLargeRoofRight3
        | ObjectName::HouseLargeDoorLeft2
        | ObjectName::HouseLargeWallMiddle2
        | ObjectName::HouseLargeDoorRight2
        | ObjectName::HouseLargeDoorMiddle2
    )
  }

  /// Returns the correct index for a non-decorative object (such as paths and buildings) sprite based on its name.
  /// Falls back to `0` for all invalid object names. Non-decorative object sprites need to be determined separately
  /// because, even though some of them (such as paths) may be on the same sprite sheet as "regular" objects, they do
  /// not have [`crate::generation::object::lib::TerrainState`]s (which themselves are derived from rule set assets)
  /// associated with them.
  /// # Panics
  /// If called on a decorative object (such as bushes, flowers, trees, etc.), this function will panic.
  pub fn get_sprite_index(&self) -> i32 {
    let index = self.get_index();

    match index {
      0 => {
        warn!("You are trying to determine the sprite index of [{:?}]", self);
        if self.is_path() {
          return 0;
        }
        panic!("You cannot determine the index of a decorative object by calling ObjectName::get_sprite_index()")
      }
      _ => self.get_index(),
    }
  }

  fn get_index(&self) -> i32 {
    match self {
      ObjectName::PathRight => 32,
      ObjectName::PathHorizontal => 33,
      ObjectName::PathCross => 34,
      ObjectName::PathVertical => 35,
      ObjectName::PathBottom => 36,
      ObjectName::PathTop => 37,
      ObjectName::PathLeft => 38,
      ObjectName::PathTopRight => 39,
      ObjectName::PathTopLeft => 40,
      ObjectName::PathBottomRight => 41,
      ObjectName::PathBottomLeft => 42,
      ObjectName::PathTopHorizontal => 43,
      ObjectName::PathBottomHorizontal => 44,
      ObjectName::PathLeftVertical => 45,
      ObjectName::PathRightVertical => 46,
      ObjectName::HouseMediumRoofLeft1 => 1,
      ObjectName::HouseMediumRoofMiddle1 => 2,
      ObjectName::HouseMediumRoofRight1 => 3,
      ObjectName::HouseMediumDoorMiddle2 => 9,
      ObjectName::HouseMediumWallLeft => 10,
      ObjectName::HouseMediumDoorMiddle1 => 11,
      ObjectName::HouseMediumWallRight => 12,
      ObjectName::HouseMediumRoofLeft2 => 19,
      ObjectName::HouseMediumRoofMiddle2 => 20,
      ObjectName::HouseMediumRoofRight2 => 21,
      ObjectName::HouseMediumDoorLeft1 => 28,
      ObjectName::HouseMediumWallMiddle1 => 29,
      ObjectName::HouseMediumDoorRight1 => 30,
      ObjectName::HouseMediumRoofLeft3 => 37,
      ObjectName::HouseMediumRoofMiddle3 => 38,
      ObjectName::HouseMediumRoofRight3 => 39,
      ObjectName::HouseMediumDoorLeft2 => 46,
      ObjectName::HouseMediumWallMiddle2 => 47,
      ObjectName::HouseMediumDoorRight2 => 48,
      ObjectName::HouseLargeRoofLeft1 => 4,
      ObjectName::HouseLargeRoofMiddle1 => 5,
      ObjectName::HouseLargeRoofRight1 => 6,
      ObjectName::HouseLargeWallLeft => 13,
      ObjectName::HouseLargeDoorMiddle1 => 14,
      ObjectName::HouseLargeWallRight => 15,
      ObjectName::HouseLargeDoorMiddle2 => 18,
      ObjectName::HouseLargeRoofLeft2 => 22,
      ObjectName::HouseLargeRoofMiddle2 => 23,
      ObjectName::HouseLargeRoofRight2 => 24,
      ObjectName::HouseLargeDoorLeft1 => 31,
      ObjectName::HouseLargeWallMiddle1 => 32,
      ObjectName::HouseLargeDoorRight1 => 33,
      ObjectName::HouseLargeRoofLeft3 => 40,
      ObjectName::HouseLargeRoofMiddle3 => 41,
      ObjectName::HouseLargeRoofRight3 => 42,
      ObjectName::HouseLargeDoorLeft2 => 49,
      ObjectName::HouseLargeWallMiddle2 => 50,
      ObjectName::HouseLargeDoorRight2 => 51,
      ObjectName::HouseSmallRoofLeft1 => 7,
      ObjectName::HouseSmallRoofRight1 => 8,
      ObjectName::HouseSmallWallLeft1 => 16,
      ObjectName::HouseSmallWallRight1 => 17,
      ObjectName::HouseSmallRoofLeft2 => 25,
      ObjectName::HouseSmallRoofRight2 => 26,
      ObjectName::HouseSmallDoorLeft => 34,
      ObjectName::HouseSmallDoorRight => 35,
      ObjectName::HouseSmallRoofLeft3 => 43,
      ObjectName::HouseSmallRoofRight3 => 44,
      ObjectName::HouseSmallWallLeft2 => 52,
      ObjectName::HouseSmallWallRight2 => 53,
      _ => 0,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use strum::IntoEnumIterator;

  #[test]
  fn get_index_for_building_variants_returns_nonzero_index() {
    for obj in ObjectName::iter() {
      if obj.is_building() {
        // If this fails, you probably forgot to update the index mapping in `get_index_for_building()`
        assert_ne!(obj.get_index(), 0, "[{:?}] returns 0 index", obj);
      }
    }
  }

  #[test]
  fn get_index_for_path_variants_returns_nonzero_index() {
    for obj in ObjectName::iter() {
      if obj.is_path() {
        if obj == ObjectName::PathUndefined {
          continue;
        }
        // If this fails, you probably forgot to update the index mapping in `get_index_for_path()`
        assert_ne!(obj.get_index(), 0, "[{:?}] returns 0 index", obj);
      }
    }
  }

  #[test]
  #[should_panic(
    expected = "You cannot determine the index of a decorative object by calling ObjectName::get_sprite_index()"
  )]
  fn get_sprite_index_for_non_path_or_building_panics() {
    let _ = ObjectName::GrassBush1.get_sprite_index();
  }
}
