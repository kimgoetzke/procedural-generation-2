use crate::coordinates::{Direction, Point};
use crate::generation::model::SettlementResources;
use crate::generation::object::model::{BuildingTemplate, ObjectName};
use crate::generation::object::settlements::FieldShape;
use bevy::asset::{Asset, Assets, Handle};
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::{Res, ResMut, Resource};
use bevy::reflect::TypePath;

/// Maps building components to their sprite variants while settlement resources are initialised.
#[derive(serde::Deserialize, Asset, TypePath, Debug, Clone, Default, PartialEq, Eq)]
pub(in crate::generation::generation_resources) struct BuildingComponentRegistry {
  components: HashMap<BuildingType, HashMap<Level, HashMap<StructureType, Vec<ObjectName>>>>,
}

impl BuildingComponentRegistry {
  fn variants_for(&self, building_type: BuildingType, level: Level, structure_type: StructureType) -> Option<&[ObjectName]> {
    self
      .components
      .get(&building_type)?
      .get(&level)?
      .get(&structure_type)
      .map(Vec::as_slice)
  }
}

#[derive(serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum BuildingType {
  SmallHouse,
  MediumHouse,
  LargeHouse,
}

#[derive(serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Level {
  GroundFloor,
  Roof,
}

#[derive(serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum StructureType {
  Left,
  Middle,
  Right,
  LeftDoor,
  MiddleDoor,
  RightDoor,
}

impl StructureType {
  const fn is_door(self) -> bool {
    matches!(self, Self::LeftDoor | Self::MiddleDoor | Self::RightDoor)
  }
}

#[derive(serde::Deserialize, Debug, Clone, PartialEq, Eq)]
struct BuildingLevelDefinition {
  level: Level,
  structures: Vec<StructureType>,
}

#[derive(serde::Deserialize, Debug, Clone, PartialEq, Eq)]
struct BuildingTemplateDefinition {
  id: String,
  building_type: BuildingType,
  door: [i32; 2],
  connection_direction: Direction,
  levels: Vec<BuildingLevelDefinition>,
}

#[derive(serde::Deserialize, Debug, Clone, PartialEq, Eq)]
struct FieldShapeDefinition {
  id: String,
  rows: Vec<String>,
}

/// Raw settlement templates loaded from TOML. Describes the shapes fields can have (excluding rotations and entrances)
/// as well as the buildings templates (sprite components).
#[derive(serde::Deserialize, Asset, TypePath, Debug, Clone, Default)]
pub(in crate::generation::generation_resources) struct SettlementTemplateAsset {
  field_shapes: Vec<FieldShapeDefinition>,
  building_templates: Vec<BuildingTemplateDefinition>,
}

#[derive(Resource, Default, Debug, Clone)]
pub(in crate::generation::generation_resources) struct SettlementTemplateAssetHandle(pub Handle<SettlementTemplateAsset>);

#[derive(Resource, Default, Debug, Clone)]
pub(in crate::generation::generation_resources) struct BuildingComponentRegistryHandle(
  pub Handle<BuildingComponentRegistry>,
);

pub(in crate::generation::generation_resources) fn populate_settlement_resources(
  settlement_resources: &mut SettlementResources,
  settlement_template_handle: &Res<SettlementTemplateAssetHandle>,
  settlement_template_assets: &mut ResMut<Assets<SettlementTemplateAsset>>,
  building_component_handle: &Res<BuildingComponentRegistryHandle>,
  building_component_assets: &mut ResMut<Assets<BuildingComponentRegistry>>,
) {
  let settlement_templates = settlement_template_assets
    .remove(&settlement_template_handle.0)
    .unwrap_or_else(|| panic!("Loaded settlement template asset is unavailable"));
  let building_components = building_component_assets
    .remove(&building_component_handle.0)
    .unwrap_or_else(|| panic!("Loaded building component asset is unavailable"));
  *settlement_resources = resolve_settlement_resources(settlement_templates, &building_components)
    .unwrap_or_else(|error| panic!("Settlement configuration is invalid: {error}"));
}

fn resolve_settlement_resources(
  templates: SettlementTemplateAsset,
  components: &BuildingComponentRegistry,
) -> Result<SettlementResources, String> {
  let mut ids = HashSet::new();
  let field_shapes = templates
    .field_shapes
    .iter()
    .map(|definition| resolve_field_shape(definition, &mut ids))
    .collect::<Result<Vec<FieldShape>, _>>()?;

  ids.clear();
  let building_templates = templates
    .building_templates
    .into_iter()
    .map(|definition| resolve_building_template(definition, components, &mut ids))
    .collect::<Result<Vec<BuildingTemplate>, _>>()?;

  if field_shapes.is_empty() {
    return Err("Settlement templates contain no field shapes".to_string());
  }
  if building_templates.is_empty() {
    return Err("Settlement templates contain no building templates".to_string());
  }

  Ok(SettlementResources::new(field_shapes, building_templates))
}

fn resolve_field_shape(definition: &FieldShapeDefinition, ids: &mut HashSet<String>) -> Result<FieldShape, String> {
  if !ids.insert(definition.id.clone()) {
    return Err(format!("Duplicate field shape id [{}]", definition.id));
  }
  let Some(width) = definition.rows.first().map(String::len) else {
    return Err(format!("Field shape [{}] has no rows", definition.id));
  };
  if width == 0 || definition.rows.iter().any(|row| row.len() != width) {
    return Err(format!("Field shape [{}] must be a non-empty rectangle", definition.id));
  }

  let mut tiles = Vec::new();
  for (y, row) in definition.rows.iter().enumerate() {
    for (x, cell) in row.bytes().enumerate() {
      match cell {
        b'#' => tiles.push((x as i32, y as i32)),
        b'.' => {}
        _ => return Err(format!("Field shape [{}] contains an unsupported character", definition.id)),
      }
    }
  }
  if tiles.is_empty() {
    return Err(format!("Field shape [{}] has no occupied tiles", definition.id));
  }

  Ok(FieldShape::new(tiles))
}

fn resolve_building_template(
  definition: BuildingTemplateDefinition,
  components: &BuildingComponentRegistry,
  ids: &mut HashSet<String>,
) -> Result<BuildingTemplate, String> {
  if !ids.insert(definition.id.clone()) {
    return Err(format!("Duplicate building template id [{}]", definition.id));
  }
  if !matches!(
    definition.connection_direction,
    Direction::Top | Direction::Right | Direction::Bottom | Direction::Left
  ) {
    return Err(format!(
      "Building template [{}] has a non-cardinal connection direction",
      definition.id
    ));
  }

  let Some(width) = definition.levels.first().map(|level| level.structures.len()) else {
    return Err(format!("Building template [{}] has no levels", definition.id));
  };
  if width == 0 || definition.levels.iter().any(|level| level.structures.len() != width) {
    return Err(format!(
      "Building template [{}] levels must have the same non-zero width",
      definition.id
    ));
  }
  let height = definition.levels.len();
  let [door_x, door_y] = definition.door;
  if door_x < 0 || door_y < 0 || door_x as usize >= width || door_y as usize >= height {
    return Err(format!("Building template [{}] door is outside its layout", definition.id));
  }
  if !definition.levels[door_y as usize].structures[door_x as usize].is_door() {
    return Err(format!(
      "Building template [{}] door position does not contain a door component",
      definition.id
    ));
  }

  let tile_variants = definition
    .levels
    .iter()
    .map(|level| {
      level
        .structures
        .iter()
        .map(|structure_type| {
          let variants = components
            .variants_for(definition.building_type, level.level, *structure_type)
            .ok_or_else(|| {
              format!(
                "Building template [{}] has no variants for [{:?}] [{:?}] [{:?}]",
                definition.id, definition.building_type, level.level, structure_type
              )
            })?;
          if variants.is_empty() || variants.iter().any(|name| !name.is_building()) {
            return Err(format!(
              "Building template [{}] has invalid variants for [{:?}] [{:?}] [{:?}]",
              definition.id, definition.building_type, level.level, structure_type
            ));
          }
          Ok(variants.to_vec())
        })
        .collect::<Result<Vec<_>, String>>()
    })
    .collect::<Result<Vec<_>, String>>()?;

  Ok(BuildingTemplate::new(
    definition.id,
    width as i32,
    height as i32,
    tile_variants,
    Point::new_internal_grid(door_x, door_y),
    definition.connection_direction,
  ))
}

#[cfg(test)]
pub(crate) fn test_settlement_resources() -> SettlementResources {
  let templates: SettlementTemplateAsset =
    toml::from_str(include_str!("../../../assets/objects/settlements/settlement-templates.toml")).unwrap();
  let components: BuildingComponentRegistry =
    toml::from_str(include_str!("../../../assets/objects/settlements/building-components.toml")).unwrap();

  resolve_settlement_resources(templates, &components).unwrap()
}

#[cfg(test)]
mod tests {
  use super::*;
  use rand::SeedableRng;
  use rand::prelude::StdRng;

  #[test]
  fn resolve_settlement_resources_resolves_configured_templates_and_components() {
    let resources = test_settlement_resources();

    assert_eq!(resources.field_shapes().len(), 8);
    assert_eq!(resources.building_templates().len(), 8);
    let medium_north = resources
      .building_templates()
      .iter()
      .find(|template| template.id == "medium_house_facing_north")
      .unwrap();
    assert_eq!((medium_north.width, medium_north.height), (3, 2));
    assert_eq!(
      medium_north.calculate_origin_ig_from_connection_point(Point::new_internal_grid(5, 5)),
      Point::new_internal_grid(4, 3)
    );
    let tiles = medium_north.generate_tiles(&mut StdRng::seed_from_u64(42));
    assert!(matches!(
      tiles[0][0],
      ObjectName::HouseMediumRoofLeft1 | ObjectName::HouseMediumRoofLeft2 | ObjectName::HouseMediumRoofLeft3
    ));
  }
}
