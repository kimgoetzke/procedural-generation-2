use crate::coords::point::{ChunkGrid, InternalGrid};
use crate::coords::Point;
use crate::generation::lib::{get_direction_points, Direction};
use bevy::app::{App, Plugin};
use bevy::log::*;
use bevy::prelude::{Reflect, ReflectResource, Resource};
use bevy::utils::HashMap;
use std::fmt::Display;
use std::ops::Range;

pub struct MetadataPlugin;

impl Plugin for MetadataPlugin {
  fn build(&self, app: &mut App) {
    app
      .init_resource::<Metadata>()
      .register_type::<Metadata>()
      .register_type::<BiomeMetadata>();
  }
}

/// This resource holds data used during world generation, providing context that spans multiple chunks. In practice,
/// data is stored in `HashMap`s with `Point<ChunkGrid>` as keys.
///
/// For example, `ElevationMetadata` is used in tile generation to ensure seamless terrain transitions across chunks
/// which allows you to configure smooth transitions from water in the west, through coastal areas and grassy plains,
/// to forests in the east.
#[derive(Resource, Default, Clone, Reflect)]
#[reflect(Resource)]
pub struct Metadata {
  pub current_chunk_cg: Point<ChunkGrid>,
  pub index: Vec<Point<ChunkGrid>>,
  pub elevation: HashMap<Point<ChunkGrid>, ElevationMetadata>,
  pub biome: HashMap<Point<ChunkGrid>, BiomeMetadata>,
}

impl Metadata {
  /// Returns the biome metadata for the given `Point<ChunkGrid>` which includes the biome metadata for the four
  /// adjacent chunks as well.
  pub fn get_biome_metadata_for(&self, cg: &Point<ChunkGrid>) -> BiomeMetadataSet {
    let bm: HashMap<Direction, &BiomeMetadata> = get_direction_points(cg)
      .iter()
      .map(|(direction, point)| {
        let metadata = self
          .biome
          .get(point)
          .expect(format!("Failed to get biome metadata for {} when retrieving data for {}", point, cg).as_str());
        (*direction, metadata)
      })
      .collect();

    let set = BiomeMetadataSet {
      top: bm.get(&Direction::Top).expect(format!("No top metadata for {}", cg).as_str()),
      top_right: bm
        .get(&Direction::TopRight)
        .expect(format!("No top right metadata for {}", cg).as_str()),
      right: bm
        .get(&Direction::Right)
        .expect(format!("No right metadata for {}", cg).as_str()),
      bottom_right: bm
        .get(&Direction::BottomRight)
        .expect(format!("No bottom right metadata for {}", cg).as_str()),
      bottom: bm
        .get(&Direction::Bottom)
        .expect(format!("No bottom metadata for {}", cg).as_str()),
      bottom_left: bm
        .get(&Direction::BottomLeft)
        .expect(format!("No bottom left metadata for {}", cg).as_str()),
      left: bm
        .get(&Direction::Left)
        .expect(format!("No left metadata for {}", cg).as_str()),
      this: bm.get(&Direction::Center).expect(format!("No metadata for {}", cg).as_str()),
      top_left: bm
        .get(&Direction::TopLeft)
        .expect(format!("No top left metadata for {}", cg).as_str()),
    };
    info!("Biome metadata for {}: {}", cg, set);
    set
  }
}

/// Metadata used to calculate an additional offset for any given `Point<InternalGrid>`. It is defined at the
/// `ChunkGrid` level and includes:
/// - `x_step`: The total elevation change applied across the x-axis of the chunk.
/// - `x`: The exact range of x-values within the chunk that achieve the specified elevation change.
/// - `y_step`: The total elevation change applied across the y-axis of the chunk.
/// - `y`: The exact range of y-values within the chunk that achieve the specified elevation change.
#[derive(Clone, Debug, Reflect)]
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

#[derive(Resource, Clone, Debug, Reflect)]
#[reflect(Resource)]
pub struct BiomeMetadata {
  pub cg: Point<ChunkGrid>,
  pub is_rocky: bool,
  pub rainfall: f32,
  pub max_layer: i32,
  pub climate: Climate,
}

impl BiomeMetadata {
  pub fn new(cg: Point<ChunkGrid>, is_rocky: bool, rainfall: f32, max_layer: i32, climate: Climate) -> Self {
    Self {
      cg,
      is_rocky,
      rainfall,
      max_layer,
      climate,
    }
  }
}

#[derive(Debug)]
pub struct BiomeMetadataSet<'a> {
  pub this: &'a BiomeMetadata,
  pub top: &'a BiomeMetadata,
  pub top_right: &'a BiomeMetadata,
  pub right: &'a BiomeMetadata,
  pub bottom_right: &'a BiomeMetadata,
  pub bottom: &'a BiomeMetadata,
  pub bottom_left: &'a BiomeMetadata,
  pub left: &'a BiomeMetadata,
  pub top_left: &'a BiomeMetadata,
}

impl Display for BiomeMetadataSet<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "\nBiome metadata set for {}: {:?}\n\
      ├─> Top left: {:?}\n\
      ├─> Top: {:?} \n\
      ├─> Top right: {:?} \n\
      ├─> Left: {:?} \n\
      ├─> Right: {:?} \n\
      ├─> Bottom left: {:?} \n\
      ├─> Bottom: {:?} \n\
      └─> Bottom right: {:?} \n",
      self.this.cg,
      self.this,
      self.top_left,
      self.top,
      self.top_right,
      self.left,
      self.right,
      self.bottom_left,
      self.bottom,
      self.bottom_right,
    )
  }
}

impl BiomeMetadataSet<'_> {
  pub fn get(&self, direction: &Direction) -> &BiomeMetadata {
    match direction {
      Direction::Top => self.top,
      Direction::Right => self.right,
      Direction::Bottom => self.bottom,
      Direction::Left => self.left,
      Direction::Center => self.this,
      _ => panic!("You requested an invalid direction for a biome metadata set"),
    }
  }

  pub fn is_same_climate(&self, direction: &Direction) -> bool {
    match direction {
      Direction::TopRight => {
        self.top.climate == self.this.climate
          && self.right.climate == self.this.climate
          && self.top_right.climate == self.this.climate
      }
      Direction::BottomRight => {
        self.right.climate == self.this.climate
          && self.bottom.climate == self.this.climate
          && self.bottom_right.climate == self.this.climate
      }
      Direction::BottomLeft => {
        self.bottom.climate == self.this.climate
          && self.left.climate == self.this.climate
          && self.bottom_left.climate == self.this.climate
      }
      Direction::TopLeft => {
        self.left.climate == self.this.climate
          && self.top.climate == self.this.climate
          && self.top_left.climate == self.this.climate
      }
      direction => self.this.climate == self.get(direction).climate,
    }
  }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Reflect)]
pub enum Climate {
  Dry,
  Moderate,
  Humid,
}

impl Climate {
  pub fn from(rainfall: f64) -> Self {
    match rainfall {
      n if n < 0.33 => Climate::Dry,
      n if n < 0.65 => Climate::Moderate,
      _ => Climate::Humid,
    }
  }
}
