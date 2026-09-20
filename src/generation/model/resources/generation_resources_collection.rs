use crate::generation::model::resources::asset_collection::AssetCollection;
use crate::generation::model::resources::asset_pack::AssetPack;
use crate::generation::model::resources::object_resources::ObjectResources;
use crate::generation::model::resources::settlement_resources::SettlementResources;
use crate::generation::model::{Climate, TerrainType};
use bevy::prelude::Resource;

/// A collection of all assets, rules, and other data that is used when spawning terrain and object sprites in the
/// world. Initialised in the [`crate::generation::generation_resources::GenerationResourcesCollectionPlugin`] on startup.
///
/// Each terrain layer and climate combination has its own [`AssetCollection`], which contains a static and optional
/// animated [`AssetPack`].
///
/// It also stores object assets, wave function collapse terrain states, and resolved settlement templates used during
/// object generation.
#[derive(Resource, Default, Debug, Clone)]
pub struct GenerationResourcesCollection {
  pub placeholder: AssetPack,
  pub water: AssetCollection,
  pub shore: AssetCollection,
  pub land_dry_l1: AssetCollection,
  pub land_dry_l2: AssetCollection,
  pub land_dry_l3: AssetCollection,
  pub land_moderate_l1: AssetCollection,
  pub land_moderate_l2: AssetCollection,
  pub land_moderate_l3: AssetCollection,
  pub land_humid_l1: AssetCollection,
  pub land_humid_l2: AssetCollection,
  pub land_humid_l3: AssetCollection,
  pub objects: ObjectResources,
  pub settlements: SettlementResources,
}

impl GenerationResourcesCollection {
  pub fn get_terrain_collection(&self, terrain: &TerrainType, climate: &Climate) -> &AssetCollection {
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

  pub fn get_object_collection(
    &self,
    terrain: TerrainType,
    climate: Climate,
    is_large_sprite: bool,
    is_settlement_structure: bool,
    is_animated: bool,
  ) -> &AssetCollection {
    if is_settlement_structure {
      return &self.objects.settlements;
    }
    if is_animated {
      return &self.objects.animated;
    }
    match (terrain, climate, is_large_sprite) {
      (TerrainType::Water, _, _) => &self.objects.water,
      (TerrainType::Shore, _, _) => &self.objects.shore,
      (TerrainType::Land1, Climate::Dry, _) => &self.objects.l1_dry,
      (TerrainType::Land1, Climate::Moderate, _) => &self.objects.l1_moderate,
      (TerrainType::Land1, Climate::Humid, _) => &self.objects.l1_humid,
      (TerrainType::Land2, Climate::Dry, _) => &self.objects.l2_dry,
      (TerrainType::Land2, Climate::Moderate, _) => &self.objects.l2_moderate,
      (TerrainType::Land2, Climate::Humid, _) => &self.objects.l2_humid,
      (TerrainType::Land3, Climate::Dry, true) => &self.objects.trees_dry,
      (TerrainType::Land3, Climate::Moderate, true) => &self.objects.trees_moderate,
      (TerrainType::Land3, Climate::Humid, true) => &self.objects.trees_humid,
      (TerrainType::Land3, Climate::Dry, _) => &self.objects.l3_dry,
      (TerrainType::Land3, Climate::Moderate, _) => &self.objects.l3_moderate,
      (TerrainType::Land3, Climate::Humid, _) => &self.objects.l3_humid,
      (TerrainType::Any, _, _) => panic!("You must not use TerrainType::Any when rendering tiles"),
    }
  }
}
