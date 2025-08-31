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
}
