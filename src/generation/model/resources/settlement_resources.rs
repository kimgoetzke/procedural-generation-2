use crate::generation::object::model::BuildingTemplate;
use crate::generation::object::settlements::FieldShape;

/// Resolved field shapes and building templates used during the settlement generation process.
#[derive(Debug, Clone, Default)]
pub struct SettlementResources {
  field_shapes: Vec<FieldShape>,
  building_templates: Vec<BuildingTemplate>,
}

impl SettlementResources {
  pub(crate) fn new(field_shapes: Vec<FieldShape>, building_templates: Vec<BuildingTemplate>) -> Self {
    Self {
      field_shapes,
      building_templates,
    }
  }

  /// Returns configured field shapes.
  pub(crate) fn field_shapes(&self) -> &[FieldShape] {
    &self.field_shapes
  }

  /// Returns configured building templates.
  pub(crate) fn building_templates(&self) -> &[BuildingTemplate] {
    &self.building_templates
  }
}
