use crate::coords::point::{ChunkGrid, InternalGrid};
use crate::coords::Point;
use crate::generation::lib::MetadataComponent;
use bevy::app::{App, Plugin};
use bevy::log::*;
use bevy::prelude::{OnAdd, OnRemove, Query, ResMut, Resource, Trigger};
use bevy::utils::HashMap;
use std::ops::Range;

pub struct MetadataPlugin;

impl Plugin for MetadataPlugin {
  fn build(&self, app: &mut App) {
    app
      .init_resource::<Metadata>()
      .init_resource::<MetadataComponentIndex>()
      .observe(on_add_metadata_component_trigger)
      .observe(on_remove_metadata_component_trigger);
  }
}

/// This resource holds data used during world generation, providing context that spans multiple chunks. In practice,
/// data is stored in `HashMap`s with `Point<ChunkGrid>` as keys.
///
/// For example, `ElevationMetadata` is used in tile generation to ensure seamless terrain transitions across chunks
/// which allows you to configure smooth transitions from water in the west, through coastal areas and grassy plains,
/// to forests in the east.
#[derive(Resource, Default, Clone)]
pub struct Metadata {
  pub elevation: HashMap<Point<ChunkGrid>, ElevationMetadata>,
}

/// Metadata used to calculate an additional offset for any given `Point<InternalGrid>`. It is defined at the
/// `ChunkGrid` level and includes:
/// - `x_step`: The total elevation change applied across the x-axis of the chunk.
/// - `x`: The exact range of x-values within the chunk that achieve the specified elevation change.
/// - `y_step`: The total elevation change applied across the y-axis of the chunk.
/// - `y`: The exact range of y-values within the chunk that achieve the specified elevation change.
#[derive(Clone, Debug)]
pub struct ElevationMetadata {
  pub x_step: f32,
  pub x: Range<f32>,
  pub y_step: f32,
  pub y: Range<f32>,
}

impl ElevationMetadata {
  /// Give it a `Point<InternalGrid>` and it will calculate the elevation offset you need to apply for that point.
  pub fn calculate_for_point(&self, ig: Point<InternalGrid>, grid_size: i32, grid_buffer: i32) -> f64 {
    self.calculate_x(ig.x as f64 + grid_buffer as f64, grid_size as f64)
      + self.calculate_y(ig.y as f64 + grid_buffer as f64, grid_size as f64)
  }

  /// Calculates the x-offset for a given x-coordinate.
  fn calculate_x(&self, coordinate: f64, grid_size: f64) -> f64 {
    self.x.start as f64 + (coordinate / grid_size) * self.x_step as f64 - self.x_step as f64 / 2.0
  }

  /// Calculates the y-offset for a given y-coordinate value. The y-axis is inverted in this application, so we need to
  /// invert the calculation as well.
  fn calculate_y(&self, coordinate: f64, grid_size: f64) -> f64 {
    self.y.end as f64 - (coordinate / grid_size) * self.y_step as f64 + self.y_step as f64 / 2.0
  }
}

#[derive(Resource, Default)]
pub struct MetadataComponentIndex {
  map: HashMap<Point<ChunkGrid>, MetadataComponent>,
}

#[allow(dead_code)]
impl MetadataComponentIndex {
  pub fn get(&self, cg: Point<ChunkGrid>) -> Option<&MetadataComponent> {
    if let Some(entity) = self.map.get(&cg) {
      Some(entity)
    } else {
      None
    }
  }
}

fn on_add_metadata_component_trigger(
  trigger: Trigger<OnAdd, MetadataComponent>,
  query: Query<&MetadataComponent>,
  mut index: ResMut<MetadataComponentIndex>,
) {
  let cc = query.get(trigger.entity()).expect("Failed to get MetadataComponent");
  index.map.insert(cc.cg, cc.clone());
  debug!("MetadataComponentIndex <- Added MetadataComponent key {:?}", cc.cg);
}

fn on_remove_metadata_component_trigger(
  trigger: Trigger<OnRemove, MetadataComponent>,
  query: Query<&MetadataComponent>,
  mut index: ResMut<MetadataComponentIndex>,
) {
  let cc = query.get(trigger.entity()).expect("Failed to get MetadataComponent");
  index.map.remove(&cc.cg);
  debug!("MetadataComponentIndex -> Removed MetadataComponent with key {:?}", cc.cg);
}
