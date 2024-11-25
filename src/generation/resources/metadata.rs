use crate::coords::point::{ChunkGrid, InternalGrid};
use crate::coords::Point;
use bevy::app::{App, Plugin};
use bevy::prelude::Resource;
use bevy::utils::HashMap;
use std::ops::Range;

pub struct MetadataPlugin;

impl Plugin for MetadataPlugin {
  fn build(&self, app: &mut App) {
    app.init_resource::<Metadata>();
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
  pub current_chunk_cg: Point<ChunkGrid>,
  pub index: Vec<Point<ChunkGrid>>,
  pub elevation: HashMap<Point<ChunkGrid>, ElevationMetadata>,
  pub biome: HashMap<Point<ChunkGrid>, BiomeMetadata>,
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

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct BiomeMetadata {
  pub is_rocky: bool,
  pub humidity: f32,
  pub max_layer: i32,
  pub biome: Biome,
}

impl BiomeMetadata {
  pub fn new(is_rocky: bool, humidity: f32, max_layer: i32, biome: Biome) -> Self {
    Self {
      is_rocky,
      humidity,
      max_layer,
      biome,
    }
  }
}

#[derive(Clone, Copy, Debug)]
pub enum Biome {
  Dry,
  Default,
  Humid,
}

impl Biome {
  pub fn from(humidity: f64) -> Self {
    match humidity {
      n if n < 0.5 => Biome::Default,
      _ => Biome::Dry,
    }
  }
}
