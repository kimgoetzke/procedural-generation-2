use crate::generation::lib::TerrainType;
use crate::generation::lib::resources::asset_collection::AssetCollection;
use crate::generation::lib::resources::asset_pack::AssetPack;
use crate::generation::lib::resources::object_resources::ObjectResources;
use crate::generation::resources::Climate;
use bevy::prelude::Resource;

#[derive(Resource, Default, Debug, Clone)]
pub struct GenerationResourcesCollection {
  pub placeholder: AssetPack,
  pub deep_water: AssetCollection,
  pub shallow_water: AssetCollection,
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
}

impl GenerationResourcesCollection {
  pub fn get_terrain_collection(&self, terrain: TerrainType, climate: Climate) -> &AssetCollection {
    match (terrain, climate) {
      (TerrainType::DeepWater, _) => &self.deep_water,
      (TerrainType::ShallowWater, _) => &self.shallow_water,
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
    is_path_sprite: bool,
  ) -> &AssetCollection {
    match (terrain, climate, is_large_sprite, is_path_sprite) {
      (TerrainType::DeepWater, _, _, true) => &self.objects.paths_water,
      (TerrainType::ShallowWater, _, _, true) => &self.objects.paths_shore,
      (TerrainType::Land1, _, _, true) => &self.objects.paths_l1,
      (TerrainType::Land2, _, _, true) => &self.objects.paths_l2,
      (TerrainType::Land3, _, _, true) => &self.objects.paths_l3,
      (TerrainType::DeepWater, _, _, _) => &self.objects.water,
      (TerrainType::ShallowWater, _, _, _) => &self.objects.shore,
      (TerrainType::Land1, Climate::Dry, _, _) => &self.objects.l1_dry,
      (TerrainType::Land1, Climate::Moderate, _, _) => &self.objects.l1_moderate,
      (TerrainType::Land1, Climate::Humid, _, _) => &self.objects.l1_humid,
      (TerrainType::Land2, Climate::Dry, _, _) => &self.objects.l2_dry,
      (TerrainType::Land2, Climate::Moderate, _, _) => &self.objects.l2_moderate,
      (TerrainType::Land2, Climate::Humid, _, _) => &self.objects.l2_humid,
      (TerrainType::Land3, Climate::Dry, true, _) => &self.objects.trees_dry,
      (TerrainType::Land3, Climate::Moderate, true, _) => &self.objects.trees_moderate,
      (TerrainType::Land3, Climate::Humid, true, _) => &self.objects.trees_humid,
      (TerrainType::Land3, Climate::Dry, _, _) => &self.objects.l3_dry,
      (TerrainType::Land3, Climate::Moderate, _, _) => &self.objects.l3_moderate,
      (TerrainType::Land3, Climate::Humid, _, _) => &self.objects.l3_humid,
      (TerrainType::Any, _, _, _) => panic!("You must not use TerrainType::Any when rendering tiles"),
    }
  }
}
