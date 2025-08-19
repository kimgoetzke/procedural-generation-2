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

  /// Returns the correct index for the path sprite based on its name. Falls back to `12` for all invalid object names.
  pub fn get_index_for_path(&self) -> i32 {
    match self {
      ObjectName::PathRight => 1,
      ObjectName::PathHorizontal => 2,
      ObjectName::PathCross => 3,
      ObjectName::PathVertical => 4,
      ObjectName::PathBottom => 5,
      ObjectName::PathTop => 6,
      ObjectName::PathLeft => 7,
      ObjectName::PathTopRight => 8,
      ObjectName::PathTopLeft => 9,
      ObjectName::PathBottomRight => 10,
      ObjectName::PathBottomLeft => 11,
      ObjectName::PathTopHorizontal => 12,
      ObjectName::PathBottomHorizontal => 13,
      ObjectName::PathLeftVertical => 14,
      ObjectName::PathRightVertical => 15,
      _ => 16,
    }
  }
}
