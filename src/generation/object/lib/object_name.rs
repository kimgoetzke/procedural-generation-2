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
  SandPathLeft,
  SandPathRight,
  SandPathTop,
  SandPathBottom,
  SandPathCross,
  SandPathHorizontal,
  SandPathVertical,
  SandPathVerticalGrassTop,
  SandPathVerticalGrassBottom,
  SandPathVerticalForestTop,
  SandPathVerticalForestBottom,
  SandPathHorizontalGrassRight,
  SandPathHorizontalGrassLeft,
  SandPathHorizontalForestRight,
  SandPathHorizontalForestLeft,
  GrassPathLeft,
  GrassPathRight,
  GrassPathTop,
  GrassPathBottom,
  GrassPathCross,
  GrassPathHorizontal,
  GrassPathVertical,
  GrassPathVerticalSandTop,
  GrassPathVerticalSandBottom,
  GrassPathVerticalForestTop,
  GrassPathVerticalForestBottom,
  GrassPathHorizontalSandRight,
  GrassPathHorizontalSandLeft,
  GrassPathHorizontalForestRight,
  GrassPathHorizontalForestLeft,
  ForestPathLeft,
  ForestPathRight,
  ForestPathTop,
  ForestPathBottom,
  ForestPathCross,
  ForestPathHorizontal,
  ForestPathVertical,
  ForestPathVerticalGrassTop,
  ForestPathVerticalGrassBottom,
  ForestPathVerticalSandTop,
  ForestPathVerticalSandBottom,
  ForestPathHorizontalGrassRight,
  ForestPathHorizontalGrassLeft,
  ForestPathHorizontalSandRight,
  ForestPathHorizontalSandLeft,
  ForestTree1,
  ForestTree2,
  ForestTree3,
  ForestTree4,
  ForestTree5,
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
}
