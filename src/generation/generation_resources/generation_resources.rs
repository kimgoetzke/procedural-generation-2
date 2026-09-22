use crate::app_states::AppState;
use crate::generation::generation_resources::settlement_asset_initialisation::{
  BuildingComponentRegistry, BuildingComponentRegistryHandle, SettlementTemplateAsset, SettlementTemplateAssetHandle,
};
use crate::generation::generation_resources::terrain_state_initialisation::{
  ExclusionsRuleSet, ExclusionsRuleSetHandle, TerrainRuleSet, TerrainRuleSetHandle, TileTypeRuleSet, TileTypeRuleSetHandle,
};
use crate::generation::generation_resources::{
  object_asset_initialisation, settlement_asset_initialisation, terrain_asset_initialisation, terrain_state_initialisation,
  terrain_state_validation,
};
use crate::generation::model::{GenerationResources, TerrainType};
use bevy::app::{App, Plugin, Startup, Update};
use bevy::asset::{AssetServer, Assets, LoadState};
use bevy::log::*;
use bevy::prelude::{Commands, IntoScheduleConfigs, NextState, OnExit, Res, ResMut, TextureAtlasLayout, in_state};
use bevy_common_assets::toml::TomlAssetPlugin;
use strum::IntoEnumIterator;

/// This plugin is responsible for loading and managing the resources - e.g. sprites and rule sets - required for the
/// generation process. The purpose of this plugin is to ensure that all necessary assets are loaded, preprocessed, and
/// initialised before the generation process starts.
///
/// At its core, this plugin adds the [`GenerationResources`] resource, making it available to the rest of the
/// application.
///
/// In terms of process, it works as follows:
/// 1. The plugin loads the rule sets for terrain and tile types from the file system. At this point, the application is
///    in the [`AppState::Loading`] state. See [`load_generation_assets_system`].
/// 2. While in this state, it checks the loading state of these assets and waits until they are fully loaded, then
///    it transitions the state to [`AppState::Initialising`]. See [`check_loading_state_system`].
/// 3. Upon transitioning to the initialising state (i.e. [`OnExit`] of [`AppState::Loading`]), it finally
///    initialises the [`GenerationResources`] resource. See [`initialise_resources_system`].
pub struct GenerationResourcesPlugin;

impl Plugin for GenerationResourcesPlugin {
  fn build(&self, app: &mut App) {
    app
      .init_resource::<GenerationResources>()
      .add_plugins((
        TomlAssetPlugin::<TerrainRuleSet>::new(&["terrain.ruleset.toml"]),
        TomlAssetPlugin::<TileTypeRuleSet>::new(&["tile-type.ruleset.toml"]),
        TomlAssetPlugin::<ExclusionsRuleSet>::new(&["exclusions.ruleset.toml"]),
        TomlAssetPlugin::<SettlementTemplateAsset>::new(&["settlement-templates.toml"]),
        TomlAssetPlugin::<BuildingComponentRegistry>::new(&["building-components.toml"]),
      ))
      .add_systems(Startup, load_generation_assets_system)
      .add_systems(Update, check_loading_state_system.run_if(in_state(AppState::Loading)))
      .add_systems(OnExit(AppState::Loading), initialise_resources_system);
  }
}

fn load_generation_assets_system(mut commands: Commands, asset_server: Res<AssetServer>) {
  let mut rule_set_handles = Vec::new();
  for terrain_type in TerrainType::iter() {
    let path = format!("objects/{}.terrain.ruleset.toml", terrain_type.to_string().to_lowercase());
    let handle = asset_server.load(path);
    rule_set_handles.push(handle);
  }
  let any_handle = asset_server.load("objects/any.terrain.ruleset.toml");
  rule_set_handles.push(any_handle);
  commands.insert_resource(TerrainRuleSetHandle(rule_set_handles));
  let all_handle = asset_server.load("objects/all.tile-type.ruleset.toml");
  commands.insert_resource(TileTypeRuleSetHandle(all_handle));
  let exclusion_handle = asset_server.load("objects/all.exclusions.ruleset.toml");
  commands.insert_resource(ExclusionsRuleSetHandle(exclusion_handle));
  let settlement_template_handle = asset_server.load("objects/settlements/settlement-templates.toml");
  commands.insert_resource(SettlementTemplateAssetHandle(settlement_template_handle));
  let building_component_handle = asset_server.load("objects/settlements/building-components.toml");
  commands.insert_resource(BuildingComponentRegistryHandle(building_component_handle));
}

fn check_loading_state_system(
  asset_server: Res<AssetServer>,
  terrain_handles: Res<TerrainRuleSetHandle>,
  tile_type_handle: Res<TileTypeRuleSetHandle>,
  exclusions_handle: Res<ExclusionsRuleSetHandle>,
  settlement_template_handle: Res<SettlementTemplateAssetHandle>,
  building_component_handle: Res<BuildingComponentRegistryHandle>,
  mut state: ResMut<NextState<AppState>>,
) {
  for handle in &terrain_handles.0 {
    if is_loading(asset_server.get_load_state(handle)) {
      info_once!("Waiting for assets to load...");
      return;
    }
  }
  if is_loading(asset_server.get_load_state(&tile_type_handle.0)) {
    info_once!("Waiting for assets to load...");
    return;
  }
  if is_loading(asset_server.get_load_state(&exclusions_handle.0))
    || is_loading(asset_server.get_load_state(&settlement_template_handle.0))
    || is_loading(asset_server.get_load_state(&building_component_handle.0))
  {
    info_once!("Waiting for assets to load...");
    return;
  }
  state.set(AppState::Initialising);
}

fn is_loading(loading_state: Option<LoadState>) -> bool {
  if let Some(state) = loading_state {
    return match state {
      LoadState::NotLoaded | LoadState::Loading => true,
      LoadState::Failed(e) => panic!("Failed to load assets: {:?}", e),
      _ => false,
    };
  };
  true
}

fn initialise_resources_system(
  asset_server: Res<AssetServer>,
  mut layouts: ResMut<Assets<TextureAtlasLayout>>,
  mut generation_resources: ResMut<GenerationResources>,
  terrain_rule_set_handle: Res<TerrainRuleSetHandle>,
  mut terrain_rule_set_assets: ResMut<Assets<TerrainRuleSet>>,
  tile_type_rule_set_handle: Res<TileTypeRuleSetHandle>,
  mut tile_type_rule_set_assets: ResMut<Assets<TileTypeRuleSet>>,
  exclusions_rule_set_handle: Res<ExclusionsRuleSetHandle>,
  mut exclusions_rule_set_assets: ResMut<Assets<ExclusionsRuleSet>>,
  settlement_template_handle: Res<SettlementTemplateAssetHandle>,
  mut settlement_template_assets: ResMut<Assets<SettlementTemplateAsset>>,
  building_component_handle: Res<BuildingComponentRegistryHandle>,
  mut building_component_assets: ResMut<Assets<BuildingComponentRegistry>>,
) {
  // Terrain sprites
  terrain_asset_initialisation::populate_terrain_assets(&mut generation_resources.world, &asset_server, &mut layouts);

  // Objects: Templates and building components for settlements
  settlement_asset_initialisation::populate_settlement_resources(
    &mut generation_resources.settlements,
    &settlement_template_handle,
    &mut settlement_template_assets,
    &building_component_handle,
    &mut building_component_assets,
  );

  // Object sprites
  object_asset_initialisation::populate_object_resources(&mut generation_resources.objects, &asset_server, &mut layouts);

  // Objects: Rule sets for wave function collapse
  let terrain_rules = terrain_state_initialisation::terrain_rules(terrain_rule_set_handle, &mut terrain_rule_set_assets);
  let tile_type_rules =
    terrain_state_initialisation::tile_type_rules(tile_type_rule_set_handle, &mut tile_type_rule_set_assets);
  let exclusion_rules =
    terrain_state_initialisation::exclusion_rules(exclusions_rule_set_handle, &mut exclusions_rule_set_assets);
  let terrain_state_map = terrain_state_initialisation::resolve_rules_to_terrain_states_map(terrain_rules, tile_type_rules);
  terrain_state_validation::validate_terrain_state_map(&terrain_state_map);
  let terrain_climate_state_map = terrain_state_initialisation::apply_exclusions(exclusion_rules, terrain_state_map);
  generation_resources
    .objects
    .set_terrain_state_climate_map(terrain_climate_state_map);
}
