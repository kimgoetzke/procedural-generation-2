use bevy::reflect::Reflect;

#[derive(serde::Deserialize, PartialEq, Debug, Clone, Copy, Reflect, Eq, Hash)]
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
  HouseSmallRoofMiddle1,
  HouseSmallRoofRight1,
  HouseSmallWallLeft,
  HouseSmallDoorMiddle,
  HouseSmallWallRight,
  HouseSmallRoofLeft2,
  HouseSmallRoofMiddle2,
  HouseSmallRoofRight2,
  HouseSmallDoorLeft1,
  HouseSmallWallMiddle1,
  HouseSmallDoorRight1,
  HouseSmallRoofLeft3,
  HouseSmallRoofMiddle3,
  HouseSmallRoofRight3,
  HouseSmallDoorLeft2,
  HouseSmallWallMiddle2,
  HouseSmallDoorRight2,
  HouseMediumRoofLeft1,
  HouseMediumRoofMiddle1,
  HouseMediumRoofRight1,
  HouseMediumWallLeft,
  HouseMediumDoorMiddle,
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
}

enum ObjectKind {
  Other,
  Path,
  Building,
}

impl ObjectName {
  pub fn get_sprite_index(&self) -> i32 {
    let object_kind = if self.is_path() {
      ObjectKind::Path
    } else if self.is_building() {
      ObjectKind::Building
    } else {
      ObjectKind::Other
    };

    match object_kind {
      ObjectKind::Path => self.get_index_for_path(),
      ObjectKind::Building => self.get_index_for_building(),
      ObjectKind::Other => {
        panic!("You cannot determine the index of a non-path/non-building object by calling ObjectName::get_sprite_index()")
      }
    }
  }

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

  /// Returns the correct index for the path sprite based on its name. Falls back to `47` for all invalid object names.
  /// Path sprites need to be determined separately because, even though on the same sprite sheet as "regular" objects,
  /// paths do not have [`crate::generation::object::lib::TerrainState`]s (which themselves are derived from rule set
  /// assets) associated with them.
  pub fn get_index_for_path(&self) -> i32 {
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
      _ => 47,
    }
  }

  pub fn is_building(&self) -> bool {
    matches!(
      self,
      ObjectName::HouseSmallRoofLeft1
        | ObjectName::HouseSmallRoofMiddle1
        | ObjectName::HouseSmallRoofRight1
        | ObjectName::HouseSmallWallLeft
        | ObjectName::HouseSmallDoorMiddle
        | ObjectName::HouseSmallWallRight
        | ObjectName::HouseSmallRoofLeft2
        | ObjectName::HouseSmallRoofMiddle2
        | ObjectName::HouseSmallRoofRight2
        | ObjectName::HouseSmallDoorLeft1
        | ObjectName::HouseSmallWallMiddle1
        | ObjectName::HouseSmallDoorRight1
        | ObjectName::HouseSmallRoofLeft3
        | ObjectName::HouseSmallRoofMiddle3
        | ObjectName::HouseSmallRoofRight3
        | ObjectName::HouseSmallDoorLeft2
        | ObjectName::HouseSmallWallMiddle2
        | ObjectName::HouseSmallDoorRight2
        | ObjectName::HouseMediumRoofLeft1
        | ObjectName::HouseMediumRoofMiddle1
        | ObjectName::HouseMediumRoofRight1
        | ObjectName::HouseMediumWallLeft
        | ObjectName::HouseMediumDoorMiddle
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
    )
  }

  pub fn get_index_for_building(&self) -> i32 {
    match self {
      ObjectName::HouseSmallRoofLeft1 => 1,
      ObjectName::HouseSmallRoofMiddle1 => 2,
      ObjectName::HouseSmallRoofRight1 => 3,
      ObjectName::HouseSmallWallLeft => 10,
      ObjectName::HouseSmallDoorMiddle => 11,
      ObjectName::HouseSmallWallRight => 12,
      ObjectName::HouseSmallRoofLeft2 => 19,
      ObjectName::HouseSmallRoofMiddle2 => 20,
      ObjectName::HouseSmallRoofRight2 => 21,
      ObjectName::HouseSmallDoorLeft1 => 28,
      ObjectName::HouseSmallWallMiddle1 => 29,
      ObjectName::HouseSmallDoorRight1 => 30,
      ObjectName::HouseSmallRoofLeft3 => 37,
      ObjectName::HouseSmallRoofMiddle3 => 38,
      ObjectName::HouseSmallRoofRight3 => 39,
      ObjectName::HouseSmallDoorLeft2 => 46,
      ObjectName::HouseSmallWallMiddle2 => 47,
      ObjectName::HouseSmallDoorRight2 => 48,
      ObjectName::HouseMediumRoofLeft1 => 4,
      ObjectName::HouseMediumRoofMiddle1 => 5,
      ObjectName::HouseMediumRoofRight1 => 6,
      ObjectName::HouseMediumWallLeft => 13,
      ObjectName::HouseMediumDoorMiddle => 14,
      ObjectName::HouseMediumWallRight => 15,
      ObjectName::HouseMediumRoofLeft2 => 22,
      ObjectName::HouseMediumRoofMiddle2 => 23,
      ObjectName::HouseMediumRoofRight2 => 24,
      ObjectName::HouseMediumDoorLeft1 => 31,
      ObjectName::HouseMediumWallMiddle1 => 32,
      ObjectName::HouseMediumDoorRight1 => 33,
      ObjectName::HouseMediumRoofLeft3 => 40,
      ObjectName::HouseMediumRoofMiddle3 => 41,
      ObjectName::HouseMediumRoofRight3 => 42,
      ObjectName::HouseMediumDoorLeft2 => 49,
      ObjectName::HouseMediumWallMiddle2 => 50,
      ObjectName::HouseMediumDoorRight2 => 51,
      _ => 0,
    }
  }
}
