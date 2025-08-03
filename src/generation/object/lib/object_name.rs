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
  PathUndefined,
}

impl ObjectName {
  pub fn is_large_sprite(&self) -> bool {
    matches!(
      self,
      ObjectName::ForestTree1
        | ObjectName::ForestTree2
        | ObjectName::ForestTree3
        | ObjectName::ForestTree4
        | ObjectName::ForestTree5
    )
  }

  pub fn is_path_sprite(&self) -> bool {
    matches!(self, ObjectName::PathUndefined)
  }
}
