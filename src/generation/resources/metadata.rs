use crate::coords::Point;
use crate::coords::point::{ChunkGrid, InternalGrid};
use crate::generation::lib::{Direction, get_direction_points};
use crate::generation::object::lib::ObjectGrid;
use bevy::app::{App, Plugin};
use bevy::log::*;
use bevy::platform::collections::HashMap;
use bevy::prelude::{Reflect, ReflectResource, Resource};
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
/// data is stored in [`HashMap`]s with [`Point<ChunkGrid>`] as keys.
///
/// For example, [`ElevationMetadata`] is used in tile generation to ensure seamless terrain transitions across chunks
/// which allows you to configure smooth transitions from water in the west, through coastal areas and grassy plains,
/// to forests in the east.
#[derive(Resource, Default, Clone, Reflect)]
#[reflect(Resource)]
pub struct Metadata {
  pub current_chunk_cg: Point<ChunkGrid>,
  /// A list of all chunk coordinates that have metadata associated with them.
  pub index: Vec<Point<ChunkGrid>>,
  /// Contains data about cross-chunk elevation changes. This influences terrain generation.
  pub elevation: HashMap<Point<ChunkGrid>, ElevationMetadata>,
  /// Contains biome metadata for each chunk such as climate. This influences which sprite sets are used.
  pub biome: HashMap<Point<ChunkGrid>, BiomeMetadata>,
  /// Contains potential connection points between chunks. This influences path generation and therefore all other
  /// object placement.
  pub connection: HashMap<Point<ChunkGrid>, Vec<Point<InternalGrid>>>,
  /// Indicates whether a chunk is considered to be settled or not. This influences the generation of buildings.
  pub settlement: HashMap<Point<ChunkGrid>, bool>,
}

impl Metadata {
  /// Returns the biome metadata for the given [`Point<ChunkGrid>`] which includes the biome metadata for the four
  /// adjacent chunks as well.
  pub fn get_biome_metadata_for(&self, cg: &Point<ChunkGrid>) -> BiomeMetadataSet<'_> {
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

    let biome_metadata_set = BiomeMetadataSet {
      top: bm[&Direction::Top],
      top_right: bm[&Direction::TopRight],
      right: bm[&Direction::Right],
      bottom_right: bm[&Direction::BottomRight],
      bottom: bm[&Direction::Bottom],
      bottom_left: bm[&Direction::BottomLeft],
      left: bm[&Direction::Left],
      this: bm[&Direction::Center],
      top_left: bm[&Direction::TopLeft],
    };
    trace!("Biome metadata for {}: {}", cg, biome_metadata_set);

    biome_metadata_set
  }

  /// Returns a list of valid connection points for the given [`Point<ChunkGrid>`] by filtering out any points that
  /// are invalid in the provided [`ObjectGrid`]. See [`ObjectGrid::is_valid_connection_point`] for the criteria.
  pub fn get_connection_points_for(&self, cg: &Point<ChunkGrid>, object_grid: &mut ObjectGrid) -> Vec<Point<InternalGrid>> {
    let mut connection_points = self
      .connection
      .get(cg)
      .expect(format!("Failed to get connection points for {}", cg).as_str())
      .iter()
      .filter(|p| {
        if let Some(cell) = object_grid.get_cell_mut(&p) {
          if cell.is_valid_connection_point() {
            // Uncomment below for debugging purposes
            // if let Some(tile_below) = &cell.tile_below {
            //   debug!("Keeping chunk {} connection point {:?} as a valid connection", cg, p,);
            //   tile_below.log();
            // }

            return true;
          }
          trace!(
            "Removing chunk {} connection point {:?} because is walkable={} & is_valid_connection_point={}",
            cg,
            p,
            cell.is_walkable(),
            cell.is_valid_connection_point()
          );
          if let Some(tile_below) = &cell.tile_below {
            tile_below.log();
          } else {
            trace!("- No tile below for connection point {:?}", p);
          }

          return false;
        }
        debug!(
          "Removing chunk {} connection point {:?} because there is no tile in the object grid",
          cg, p
        );

        false
      })
      .cloned()
      .collect::<Vec<_>>();

    if connection_points.len() == 1 {
      if !connection_points[0].is_touching_edge() {
        connection_points.clear();
      }
    }

    connection_points
  }

  pub fn get_settlement_status_for(&self, cg: &Point<ChunkGrid>) -> bool {
    *self.settlement.get(cg).unwrap_or(&false)
  }
}

/// Metadata used to calculate an additional offset for any given [`Point<InternalGrid>`]. It is defined at the
/// [`ChunkGrid`] level and includes:
/// - `x_step`: The total elevation change applied across the x-axis of the chunk.
/// - `x_range`: The exact range of x-values within the chunk that achieve the specified elevation change.
/// - `y_step`: The total elevation change applied across the y-axis of the chunk.
/// - `y_range`: The exact range of y-values within the chunk that achieve the specified elevation change.
///
/// The [`ElevationMetadata::is_enabled`] flag indicates whether elevation metadata is enabled or disabled, which
/// can be done via the settings, for the chunk.
#[derive(Clone, Debug, Reflect)]
pub struct ElevationMetadata {
  pub is_enabled: bool,
  pub x_step: f64,
  pub x_range: Range<f64>,
  pub y_step: f64,
  pub y_range: Range<f64>,
}

impl Display for ElevationMetadata {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "ElevationMetadata {{ {}x[{:?} at {:.5}], y[{:?} at {:.5}] }}",
      if self.is_enabled { "" } else { "DISABLED, " },
      self.x_range,
      self.x_step,
      self.y_range,
      self.y_step,
    )
  }
}

impl ElevationMetadata {
  /// Give it a [`Point<InternalGrid>`] and it will calculate the elevation offset you need to apply for that point.
  pub fn calculate_for_point(&self, ig: Point<InternalGrid>) -> f64 {
    if !self.is_enabled {
      return 0.0;
    }
    self.calculate_x(ig.x as f64) + self.calculate_y(ig.y as f64)
  }

  /// Calculates the x-offset for a given x-coordinate.
  fn calculate_x(&self, coordinate: f64) -> f64 {
    let min = self.x_range.start.min(self.x_range.end);
    let max = self.x_range.start.max(self.x_range.end);
    (self.x_range.start + (coordinate * self.x_step) - self.x_step).clamp(min, max)
  }

  /// Calculates the y-offset for a given y-coordinate value. The y-axis is inverted in this application, so we need to
  /// invert the calculation as well.
  fn calculate_y(&self, coordinate: f64) -> f64 {
    let min = self.y_range.start.min(self.y_range.end);
    let max = self.y_range.start.max(self.y_range.end);
    (self.y_range.end - (coordinate * self.y_step) + self.y_step).clamp(min, max)
  }
}

#[derive(Resource, Clone, Debug, Reflect)]
#[reflect(Resource)]
pub struct BiomeMetadata {
  pub cg: Point<ChunkGrid>,
  pub climate: Climate,
}

impl BiomeMetadata {
  pub fn new(cg: Point<ChunkGrid>, climate: Climate) -> Self {
    Self { cg, climate }
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
      Direction::TopLeft => self.top_left,
      Direction::Top => self.top,
      Direction::TopRight => self.top_right,
      Direction::Left => self.left,
      Direction::Center => self.this,
      Direction::Right => self.right,
      Direction::BottomLeft => self.bottom_left,
      Direction::Bottom => self.bottom,
      Direction::BottomRight => self.bottom_right,
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

#[cfg(test)]
mod tests {
  use super::*;

  impl Metadata {
    pub fn default(current_chunk_cg: Point<ChunkGrid>) -> Self {
      Self {
        current_chunk_cg,
        index: vec![],
        elevation: HashMap::new(),
        biome: HashMap::new(),
        connection: HashMap::new(),
        settlement: HashMap::new(),
      }
    }
  }

  #[test]
  fn get_biome_metadata_for_retrieves_biome_metadata_for_all_directions() {
    let mut metadata = Metadata::default(Point::new_chunk_grid(0, 0));
    let cg = Point::new_chunk_grid(0, 0);

    // Given some random metadata
    for (direction, point) in get_direction_points(&cg) {
      metadata
        .biome
        .insert(point.clone(), BiomeMetadata::new(point, Climate::Moderate));
      if direction == Direction::Top {
        metadata.biome.insert(point.clone(), BiomeMetadata::new(point, Climate::Dry));
      } else if direction == Direction::BottomLeft {
        metadata
          .biome
          .insert(point.clone(), BiomeMetadata::new(point, Climate::Humid));
      }
    }

    let result = metadata.get_biome_metadata_for(&cg);

    assert_eq!(result.this.cg, cg);
    assert_eq!(result.top_right.climate, Climate::Moderate);
    assert_eq!(result.top.cg, Point::new(0, 1));
    assert_eq!(result.bottom_left.cg, Point::new(-1, -1));
    assert_eq!(result.bottom_left.climate, Climate::Humid);
  }

  #[test]
  #[should_panic(expected = "Failed to get biome metadata for cg(-1, 1) when retrieving data for cg(0, 0)")]
  fn get_biome_metadata_for_panics_when_biome_metadata_is_missing_for_a_direction() {
    let mut metadata = Metadata::default(Point::new_chunk_grid(0, 0));
    let cg = Point::new_chunk_grid(0, 0);

    // Given incomplete metadata
    metadata
      .biome
      .insert(cg.clone(), BiomeMetadata::new(cg.clone(), Climate::Moderate));

    metadata.get_biome_metadata_for(&cg);
  }

  #[test]
  #[should_panic(expected = "Failed to get connection points for cg(0, 0)")]
  fn get_valid_connection_points_panics_when_there_are_no_points_for_cg() {
    let cg = Point::new_chunk_grid(0, 0);
    let mut object_grid = ObjectGrid::default(cg);
    let metadata = Metadata::default(cg);
    metadata.get_connection_points_for(&cg, &mut object_grid);
  }

  #[test]
  fn get_valid_connection_points_returns_empty_list_when_no_connection_points_exist() {
    let cg = Point::new_chunk_grid(0, 0);
    let mut object_grid = ObjectGrid::default(cg);
    let mut metadata = Metadata::default(cg);
    metadata.connection.insert(cg.clone(), vec![]);
    let result = metadata.get_connection_points_for(&cg, &mut object_grid);
    assert!(result.is_empty());
  }

  #[test]
  fn get_valid_connection_points_filters_out_non_walkable_connection_points() {
    let cg = Point::new_chunk_grid(0, 0);
    let mut object_grid = ObjectGrid::default(cg);
    if let Some(cell) = object_grid.get_cell_mut(&Point::new_internal_grid(1, 1)) {
      cell.calculate_is_walkable(); // Point (1, 1) is not walkable
    }
    let mut metadata = Metadata::default(cg);
    metadata.connection.insert(cg.clone(), vec![Point::new_internal_grid(1, 1)]);
    let result = metadata.get_connection_points_for(&cg, &mut object_grid);
    assert_eq!(result, vec![]);
  }

  #[test]
  fn get_valid_connection_points_returns_valid_connection_points() {
    let cg = Point::new_chunk_grid(0, 0);
    let mut object_grid = ObjectGrid::default_walkable(cg);
    let mut metadata = Metadata::default(cg);
    let expected_point1 = Point::new_internal_grid(1, 1);
    let expected_point2 = Point::new_internal_grid(1, 2);
    metadata.connection.insert(cg.clone(), vec![expected_point1, expected_point2]);
    let result = metadata.get_connection_points_for(&cg, &mut object_grid);
    assert_eq!(result, vec![expected_point1, expected_point2]);
  }

  #[test]
  fn calculate_for_point_is_zero_offset_when_elevation_is_disabled() {
    let elevation_metadata = ElevationMetadata {
      is_enabled: false,
      x_step: 1.0,
      x_range: 0.0..10.0,
      y_step: 1.0,
      y_range: 0.0..10.0,
    };
    let ig = Point::new_internal_grid(5, 5);
    let result = elevation_metadata.calculate_for_point(ig);
    assert_eq!(result, 0.0);
  }

  #[test]
  fn calculate_for_point_calculates_correct_offset_for_point_within_range() {
    let elevation_metadata = ElevationMetadata {
      is_enabled: true,
      x_step: 0.01,
      x_range: 0.0..5.,
      y_step: 0.01,
      y_range: 0.0..5.,
    };
    let ig = Point::new_internal_grid(4, 6);
    let result = elevation_metadata.calculate_for_point(ig);
    assert_eq!(result, 4.98);
  }

  #[test]
  fn calculate_for_point_clamps_offset_to_range_limits() {
    let elevation_metadata = ElevationMetadata {
      is_enabled: true,
      x_step: 0.5,
      x_range: 0.0..10.,
      y_step: 0.3,
      y_range: 0.0..10.,
    };
    let ig = Point::new_internal_grid(4, 6);
    let result = elevation_metadata.calculate_for_point(ig);
    assert_eq!(result, 10.);
  }

  #[test]
  fn calculates_correct_offset_for_negative_coordinates() {
    let elevation_metadata = ElevationMetadata {
      is_enabled: true,
      x_step: 0.5,
      x_range: -5.0..5.0,
      y_step: 0.3,
      y_range: -5.0..5.0,
    };
    let ig = Point::new_internal_grid(1, 6);
    let result = elevation_metadata.calculate_for_point(ig);
    assert_eq!(result, -1.5);
  }
}
