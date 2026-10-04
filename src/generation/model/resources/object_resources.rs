use crate::generation::model::resources::sprite_sheet_set::SpriteSheetSet;
use crate::generation::model::{Climate, TerrainType, TileType};
use crate::generation::object::model::TerrainState;
use bevy::platform::collections::HashMap;
use bevy::prelude::Resource;

#[derive(Resource, Default, Debug, Clone)]
pub struct ObjectResources {
  terrain_climate_state_map: HashMap<(TerrainType, Climate), HashMap<TileType, Vec<TerrainState>>>,
  pub water: SpriteSheetSet,
  pub shore: SpriteSheetSet,
  pub l1_dry: SpriteSheetSet,
  pub l1_moderate: SpriteSheetSet,
  pub l1_humid: SpriteSheetSet,
  pub l2_dry: SpriteSheetSet,
  pub l2_moderate: SpriteSheetSet,
  pub l2_humid: SpriteSheetSet,
  pub l3_dry: SpriteSheetSet,
  pub l3_moderate: SpriteSheetSet,
  pub l3_humid: SpriteSheetSet,
  pub animated: SpriteSheetSet,
  pub trees_dry: SpriteSheetSet,
  pub trees_moderate: SpriteSheetSet,
  pub trees_humid: SpriteSheetSet,
  pub settlements: SpriteSheetSet,
}

impl ObjectResources {
  /// Replaces the object-placement states indexed by terrain, climate, and tile type.
  pub fn set_terrain_state_climate_map(
    &mut self,
    map: HashMap<(TerrainType, Climate), HashMap<TileType, Vec<TerrainState>>>,
  ) {
    self.terrain_climate_state_map = map;
  }

  /// Returns object-placement states, optionally excluding animated objects.
  pub fn terrain_state_collection(
    &self,
    enable_animated_objects: bool,
  ) -> HashMap<(TerrainType, Climate), HashMap<TileType, Vec<TerrainState>>> {
    let mut terrain_climate_state_map = self.terrain_climate_state_map.clone();
    for map in terrain_climate_state_map.values_mut() {
      for states in map.values_mut() {
        states.retain(|state| enable_animated_objects || !state.name.is_animated());
      }
    }
    terrain_climate_state_map
  }

  /// Returns the sprite sheet set for an object and its terrain context.
  pub fn sprite_sheet_set(
    &self,
    terrain: TerrainType,
    climate: Climate,
    is_large_sprite: bool,
    is_settlement_structure: bool,
    is_animated: bool,
  ) -> &SpriteSheetSet {
    if is_settlement_structure {
      return &self.settlements;
    }
    if is_animated {
      return &self.animated;
    }
    match (terrain, climate, is_large_sprite) {
      (TerrainType::Water, _, _) => &self.water,
      (TerrainType::Shore, _, _) => &self.shore,
      (TerrainType::Land1, Climate::Dry, _) => &self.l1_dry,
      (TerrainType::Land1, Climate::Moderate, _) => &self.l1_moderate,
      (TerrainType::Land1, Climate::Humid, _) => &self.l1_humid,
      (TerrainType::Land2, Climate::Dry, _) => &self.l2_dry,
      (TerrainType::Land2, Climate::Moderate, _) => &self.l2_moderate,
      (TerrainType::Land2, Climate::Humid, _) => &self.l2_humid,
      (TerrainType::Land3, Climate::Dry, true) => &self.trees_dry,
      (TerrainType::Land3, Climate::Moderate, true) => &self.trees_moderate,
      (TerrainType::Land3, Climate::Humid, true) => &self.trees_humid,
      (TerrainType::Land3, Climate::Dry, _) => &self.l3_dry,
      (TerrainType::Land3, Climate::Moderate, _) => &self.l3_moderate,
      (TerrainType::Land3, Climate::Humid, _) => &self.l3_humid,
      (TerrainType::Any, _, _) => panic!("You must not use TerrainType::Any when rendering tiles"),
    }
  }
}
