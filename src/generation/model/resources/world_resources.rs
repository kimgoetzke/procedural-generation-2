use crate::generation::model::resources::sprite_sheet::SpriteSheet;
use crate::generation::model::resources::sprite_sheet_set::SpriteSheetSet;
use crate::generation::model::{Climate, TerrainType};

/// Stores sprite sheets used to render generated terrain.
///
/// Each terrain layer and climate combination has its own [`SpriteSheetSet`], which contains a static and optional
/// animated [`SpriteSheet`]s.
#[derive(Default, Debug, Clone)]
pub struct WorldResources {
  pub placeholder: SpriteSheet,
  pub water: SpriteSheetSet,
  pub shore: SpriteSheetSet,
  pub land_dry_l1: SpriteSheetSet,
  pub land_dry_l2: SpriteSheetSet,
  pub land_dry_l3: SpriteSheetSet,
  pub land_moderate_l1: SpriteSheetSet,
  pub land_moderate_l2: SpriteSheetSet,
  pub land_moderate_l3: SpriteSheetSet,
  pub land_humid_l1: SpriteSheetSet,
  pub land_humid_l2: SpriteSheetSet,
  pub land_humid_l3: SpriteSheetSet,
}

impl WorldResources {
  /// Returns the sprite sheet set for the given terrain and climate combination.
  pub fn sprite_sheet_set(&self, terrain: &TerrainType, climate: &Climate) -> &SpriteSheetSet {
    match (terrain, climate) {
      (TerrainType::Water, _) => &self.water,
      (TerrainType::Shore, _) => &self.shore,
      (TerrainType::Land1, Climate::Dry) => &self.land_dry_l1,
      (TerrainType::Land1, Climate::Moderate) => &self.land_moderate_l1,
      (TerrainType::Land1, Climate::Humid) => &self.land_humid_l1,
      (TerrainType::Land2, Climate::Dry) => &self.land_dry_l2,
      (TerrainType::Land2, Climate::Moderate) => &self.land_moderate_l2,
      (TerrainType::Land2, Climate::Humid) => &self.land_humid_l2,
      (TerrainType::Land3, Climate::Dry) => &self.land_dry_l3,
      (TerrainType::Land3, Climate::Moderate) => &self.land_moderate_l3,
      (TerrainType::Land3, Climate::Humid) => &self.land_humid_l3,
      (TerrainType::Any, _) => panic!("You must not use TerrainType::Any when rendering tiles"),
    }
  }
}
