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

#[derive(Resource, Default, Clone)]
pub struct Metadata {
  pub elevation: HashMap<Point<ChunkGrid>, ElevationMetadata>,
}

#[derive(Clone, Debug)]
pub struct ElevationMetadata {
  pub x: Range<f32>,
  pub y: Range<f32>,
  pub x_step: f32,
  pub y_step: f32,
}

impl ElevationMetadata {
  pub fn calculate_for_point(&self, ig: Point<InternalGrid>, grid_size: i32, grid_buffer: i32) -> f64 {
    self.calculate_x(ig.x as f64 + grid_buffer as f64, grid_size as f64)
      + self.calculate_y(ig.y as f64 + grid_buffer as f64, grid_size as f64)
  }

  fn calculate_x(&self, axis: f64, grid_size: f64) -> f64 {
    self.x.start as f64 + (axis / grid_size) * self.x_step as f64 - self.x_step as f64 / 2.0
  }

  fn calculate_y(&self, axis: f64, grid_size: f64) -> f64 {
    self.y.start as f64 + (axis / grid_size) * self.y_step as f64 - self.y_step as f64 / 2.0
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
