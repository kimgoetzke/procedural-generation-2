use crate::coordinates::point::InternalGrid;
use crate::coordinates::{Direction, Point};
use crate::generation::object::model::{BuildingTemplate, ObjectGrid, ObjectName};
use crate::generation::object::settlements::settlement_generation::{
  select_fitting_building, update_path_in_front_of_entrance,
};
use bevy::log::*;
use bevy::platform::collections::HashSet;
use rand::prelude::StdRng;
use rand::seq::SliceRandom;
use rand::{RngExt, SeedableRng};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FieldType {
  Wheat,
  Pasture,
}

impl FieldType {
  /// Returns the [`ObjectName`] for this field type and tile shape. Different fields share the same shapes but use
  /// different object names.
  const fn object_name(self, tile_shape: TileShape) -> ObjectName {
    match self {
      Self::Wheat => WHEAT_TILES[tile_shape as usize],
      Self::Pasture => PASTURE_TILES[tile_shape as usize],
    }
  }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
enum TileShape {
  Fill,
  EdgeTop,
  EdgeRight,
  EdgeBottom,
  EdgeLeft,
  OuterCornerTopLeft,
  OuterCornerTopRight,
  OuterCornerBottomRight,
  OuterCornerBottomLeft,
  InnerCornerTopLeft,
  InnerCornerTopRight,
  InnerCornerBottomRight,
  InnerCornerBottomLeft,
}

const WHEAT_TILES: [ObjectName; 13] = [
  ObjectName::WheatFieldFill,
  ObjectName::WheatFieldSideTop,
  ObjectName::WheatFieldSideRight,
  ObjectName::WheatFieldSideBottom,
  ObjectName::WheatFieldSideLeft,
  ObjectName::WheatFieldOuterCornerTopLeft,
  ObjectName::WheatFieldOuterCornerTopRight,
  ObjectName::WheatFieldOuterCornerBottomRight,
  ObjectName::WheatFieldOuterCornerBottomLeft,
  ObjectName::WheatFieldInnerCornerTopLeft,
  ObjectName::WheatFieldInnerCornerTopRight,
  ObjectName::WheatFieldInnerCornerBottomRight,
  ObjectName::WheatFieldInnerCornerBottomLeft,
];

const PASTURE_TILES: [ObjectName; 13] = [
  ObjectName::PastureFill,
  ObjectName::PastureSideTop,
  ObjectName::PastureSideRight,
  ObjectName::PastureSideBottom,
  ObjectName::PastureSideLeft,
  ObjectName::PastureOuterCornerTopLeft,
  ObjectName::PastureOuterCornerTopRight,
  ObjectName::PastureOuterCornerBottomRight,
  ObjectName::PastureOuterCornerBottomLeft,
  ObjectName::PastureInnerCornerTopLeft,
  ObjectName::PastureInnerCornerTopRight,
  ObjectName::PastureInnerCornerBottomRight,
  ObjectName::PastureInnerCornerBottomLeft,
];

const CARDINAL_DIRECTIONS: [Direction; 4] = [Direction::Top, Direction::Right, Direction::Bottom, Direction::Left];

const DIAGONALS: [(Direction, TileShape); 4] = [
  (Direction::TopLeft, TileShape::InnerCornerTopLeft),
  (Direction::TopRight, TileShape::InnerCornerTopRight),
  (Direction::BottomRight, TileShape::InnerCornerBottomRight),
  (Direction::BottomLeft, TileShape::InnerCornerBottomLeft),
];

type Shape = Vec<(i32, i32)>;

/// A pre-configured field e.g. an L-shaped or rectangle layout. Similar to a building template but for fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FieldShape {
  tiles: Shape,
}

impl FieldShape {
  pub(crate) const fn new(tiles: Shape) -> Self {
    Self { tiles }
  }

  pub(crate) fn tiles(&self) -> &[(i32, i32)] {
    &self.tiles
  }
}

struct FieldLayout {
  tiles: Vec<(Point<InternalGrid>, TileShape)>,
  entrance: Point<InternalGrid>,
}

/// Generates fenced fields and places them beside settlement roads. Fields must stay on one elevation, and placement
/// must leave space for at least one building because fields without buildings nearby wouldn't be very credible.
pub(super) fn place_fields(
  grid: &mut ObjectGrid,
  path_points: &[Point<InternalGrid>],
  available_grid_space: &HashSet<Point<InternalGrid>>,
  building_templates: &[BuildingTemplate],
  field_shapes: &[FieldShape],
  occupied_grid_space: &mut HashSet<Point<InternalGrid>>,
  rng: &mut StdRng,
) -> i8 {
  let capacity = estimated_building_capacity(
    path_points,
    available_grid_space,
    occupied_grid_space.clone(),
    building_templates,
  );
  let mut candidate_connection_igs = path_points.to_vec();
  candidate_connection_igs.shuffle(rng);
  let mut fields_placed = 0;

  for field_type in permitted_field_types(rng, capacity) {
    // Try every shape, orientation, and entrance until one fits
    'placement_loop: for candidate_layout in layouts(rng, field_shapes) {
      for &connection_ig in &candidate_connection_igs {
        // Skip if the chosen field layout cannot be placed
        let candidate_field = absolute_tiles(&candidate_layout, connection_ig, field_type);
        if !can_place_field(
          grid,
          &candidate_field,
          path_points,
          available_grid_space,
          occupied_grid_space,
          building_templates,
        ) {
          continue;
        }

        // ...otherwise place it
        for (field_ig, name) in &candidate_field {
          if let Some(cell) = grid.get_cell_mut(field_ig) {
            cell.mark_as_collapsed(*name);
          }
        }
        occupied_grid_space.extend(candidate_field.iter().map(|(point, _)| *point));
        let entrance = Point::new_internal_grid(
          connection_ig.x + candidate_layout.entrance.x,
          connection_ig.y + candidate_layout.entrance.y,
        );
        update_path_in_front_of_entrance(&connection_ig, &entrance, grid);
        fields_placed += 1;
        trace!(
          "Placed [{:?}] field with [{}] tiles beside {} on {}",
          field_type,
          candidate_field.len(),
          connection_ig,
          grid.cg,
        );
        break 'placement_loop;
      }
    }
  }

  fields_placed
}

/// Converts a layout relative to a path connection point into absolute internal grid points and object names. Both the
/// placement checks and the [`ObjectGrid`] operate on absolute positions.
fn absolute_tiles(
  layout: &FieldLayout,
  path_point: Point<InternalGrid>,
  field_type: FieldType,
) -> Vec<(Point<InternalGrid>, ObjectName)> {
  layout
    .tiles
    .iter()
    .map(|(offset, tile_shape)| {
      (
        Point::new_internal_grid(path_point.x + offset.x, path_point.y + offset.y),
        field_type.object_name(*tile_shape),
      )
    })
    .collect()
}

/// Checks that all proposed field tiles are available, on the same elevation, and leave room for a building. This avoids
/// partial fields, fields crossing terrain levels, and settlements containing fields but no buildings.
fn can_place_field(
  grid: &ObjectGrid,
  proposed_field_tiles: &[(Point<InternalGrid>, ObjectName)],
  path_points: &[Point<InternalGrid>],
  available_grid_space: &HashSet<Point<InternalGrid>>,
  occupied_grid_space: &HashSet<Point<InternalGrid>>,
  building_templates: &[BuildingTemplate],
) -> bool {
  if !proposed_field_tiles
    .iter()
    .all(|(point, _)| available_grid_space.contains(point) && !occupied_grid_space.contains(point))
  {
    return false;
  }
  let Some((first_point, _)) = proposed_field_tiles.first() else {
    return false;
  };
  let terrain = grid.get_cell(first_point).map(|cell| cell.terrain());
  if !proposed_field_tiles
    .iter()
    .all(|(point, _)| grid.get_cell(point).map(|cell| cell.terrain()) == terrain)
  {
    return false;
  }
  let mut reserved_grid_space = occupied_grid_space.clone();
  reserved_grid_space.extend(proposed_field_tiles.iter().map(|(point, _)| *point));
  estimated_building_capacity(path_points, available_grid_space, reserved_grid_space, building_templates) > 0
}

/// Estimates how many non-overlapping buildings fit along the available path points. Field selection and placement use
/// this estimate to preserve space for buildings.
fn estimated_building_capacity(
  path_points: &[Point<InternalGrid>],
  available_grid_space: &HashSet<Point<InternalGrid>>,
  mut reserved_grid_space: HashSet<Point<InternalGrid>>,
  building_templates: &[BuildingTemplate],
) -> usize {
  let mut rng = StdRng::seed_from_u64(0);
  let mut estimated_building_count = 0;
  for &point in path_points {
    if let Some(template) = select_fitting_building(
      building_templates,
      point,
      available_grid_space,
      &reserved_grid_space,
      &mut rng,
    ) {
      let origin = template.calculate_origin_ig_from_connection_point(point);
      for y in 0..template.height {
        for x in 0..template.width {
          reserved_grid_space.insert(Point::new_internal_grid(origin.x + x, origin.y + y));
        }
      }
      estimated_building_count += 1;
    }
  }
  estimated_building_count
}

/// Returns which field type may be placed based on the estimated building capacity. Small settlements get at most one
/// type, while larger settlements can contain both.
fn permitted_field_types(rng: &mut StdRng, capacity: usize) -> Vec<FieldType> {
  match capacity {
    0 => Vec::new(),
    1..=2 => vec![if rng.random_bool(0.5) {
      FieldType::Wheat
    } else {
      FieldType::Pasture
    }],
    _ => vec![FieldType::Wheat, FieldType::Pasture],
  }
}

/// Returns every supported field layout in randomised preference order. One random rotation per shape is tried first,
/// while the remaining rotations allow placement where the preferred rotations do not fit.
fn layouts(rng: &mut StdRng, field_shapes: &[FieldShape]) -> Vec<FieldLayout> {
  // Get configured shape templates and shuffle them
  let mut shapes: Vec<Shape> = field_shapes.iter().map(|shape| shape.tiles().to_vec()).collect();
  shapes.shuffle(rng);

  // Keep preferred and fallback layouts separate for now
  let mut preferred_layouts: Vec<FieldLayout> = Vec::new();
  let mut fallback_layouts: Vec<FieldLayout> = Vec::new();

  // Iterate through each shape
  for shape in shapes {
    // Generate every orientation, pick one as the preferred orientation, and make it the first in the list
    let mut rotated_shapes = rotations(&shape);
    let preferred_rotation = rng.random_range(0..rotated_shapes.len());
    rotated_shapes.rotate_left(preferred_rotation);

    // Iterate through each rotation of a shape
    for (rotation_index, rotated_shape) in rotated_shapes.into_iter().enumerate() {
      // Determine possible entrances for the shape
      let mut candidate_entrances = entrances(&rotated_shape);
      candidate_entrances.shuffle(rng);

      // Make sure we keep our preferred orientation layouts separate
      let layouts = if rotation_index == 0 {
        &mut preferred_layouts
      } else {
        &mut fallback_layouts
      };

      // Now add a "fully-qualified" (shape + entrance) layout for each possible entrance to the list
      layouts.extend(
        candidate_entrances
          .into_iter()
          .map(|(entrance, outside)| layout_from_entrance(&rotated_shape, entrance, outside)),
      );
    }
  }

  // Merge preferred and fallback layouts again in the correct order and return them
  preferred_layouts.extend(fallback_layouts);
  preferred_layouts
}

/// Returns all four rotations of a field shape. Trying each rotation prevents a valid placement being missed because
/// another orientation was selected first.
fn rotations(shape: &[(i32, i32)]) -> Vec<Shape> {
  let mut rotations = Vec::with_capacity(4);
  let mut rotated = shape.to_vec();
  for _ in 0..4 {
    rotations.push(rotated.clone());
    rotated = rotated.into_iter().map(|(x, y)| (-y, x)).collect();
  }
  rotations
}

/// Returns field tiles with exactly one side outside the shape. These tiles can face a path without creating two
/// openings in the fence. The result is a list of candidate entrances to a field.
fn entrances(shape: &[(i32, i32)]) -> Vec<((i32, i32), Direction)> {
  shape
    .iter()
    .filter_map(|&(x, y)| {
      let outside: Vec<Direction> = outside(shape, &x, &y);
      match outside.as_slice() {
        [direction] => Some(((x, y), *direction)),
        _ => None,
      }
    })
    .collect()
}

// TODO: Consider if we need dedicated artwork for this
/// Returns the full [`FieldLayout`]. Positions a field shape relative to a path connection point and marks the
/// connected field tile as fill. The fill tile leaves an opening in the fence between the field and path.
fn layout_from_entrance(shape: &[(i32, i32)], entrance: (i32, i32), outside: Direction) -> FieldLayout {
  let outside_offset: Point<InternalGrid> = outside.to_point();
  let tiles = shape
    .iter()
    .map(|&(x, y)| {
      let tile_shape = if (x, y) == entrance {
        TileShape::Fill
      } else {
        classify_tile(shape, x, y)
      };
      (
        Point::new_internal_grid(x - entrance.0 - outside_offset.x, y - entrance.1 - outside_offset.y),
        tile_shape,
      )
    })
    .collect();

  FieldLayout {
    tiles,
    entrance: Point::new_internal_grid(-outside_offset.x, -outside_offset.y),
  }
}

/// Returns the tile shape required by its neighbouring field tiles. The result determines which fence edges and corners
/// appear at this position.
fn classify_tile(shape: &[(i32, i32)], x: i32, y: i32) -> TileShape {
  let outside: Vec<Direction> = outside(shape, &x, &y);

  match outside.as_slice() {
    [] => classify_fill_or_inner_corner(shape, x, y),
    [Direction::Top] => TileShape::EdgeTop,
    [Direction::Right] => TileShape::EdgeRight,
    [Direction::Bottom] => TileShape::EdgeBottom,
    [Direction::Left] => TileShape::EdgeLeft,
    [Direction::Top, Direction::Right] => TileShape::OuterCornerTopRight,
    [Direction::Top, Direction::Left] => TileShape::OuterCornerTopLeft,
    [Direction::Right, Direction::Bottom] => TileShape::OuterCornerBottomRight,
    [Direction::Bottom, Direction::Left] => TileShape::OuterCornerBottomLeft,
    _ => panic!("Field shape has an unsupported one-tile-wide section at ({x}, {y})"),
  }
}

/// Checks diagonal neighbours when field tiles exist on every cardinal side. A missing diagonal requires an
/// inner-corner fence instead of a fill tile.
fn classify_fill_or_inner_corner(shape: &[(i32, i32)], x: i32, y: i32) -> TileShape {
  let missing_diagonals: Vec<_> = DIAGONALS
    .iter()
    .filter_map(|&(direction, tile_shape)| {
      let offset: Point<InternalGrid> = direction.to_point();
      (!shape.contains(&(x + offset.x, y + offset.y))).then_some(tile_shape)
    })
    .collect();

  match missing_diagonals.as_slice() {
    [] => TileShape::Fill,
    [tile_shape] => *tile_shape,
    _ => panic!("Field shape has multiple missing diagonals at ({x}, {y})"),
  }
}

/// Returns the cardinal directions without an adjacent field tile. Entrance detection and fence classification use this
/// same neighbour rule.
fn outside(shape: &[(i32, i32)], x: &i32, y: &i32) -> Vec<Direction> {
  CARDINAL_DIRECTIONS
    .into_iter()
    .filter(|direction| {
      let offset: Point<InternalGrid> = direction.to_point();
      !shape.contains(&(x + offset.x, y + offset.y))
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::generation::object::settlements::test_settlement_resources;
  use rand::SeedableRng;

  fn rectangle(width: i32, height: i32) -> Shape {
    (0..height).flat_map(|y| (0..width).map(move |x| (x, y))).collect()
  }

  #[test]
  fn rotations_include_all_four_orientations() {
    let shape = vec![(0, 0), (1, 0), (0, 1)];

    assert_eq!(
      rotations(&shape),
      vec![
        vec![(0, 0), (1, 0), (0, 1)],
        vec![(0, 0), (0, 1), (-1, 0)],
        vec![(0, 0), (-1, 0), (0, -1)],
        vec![(0, 0), (0, -1), (1, 0)],
      ]
    );
  }

  #[test]
  fn layouts_mark_the_boundary_tile_next_to_the_road_as_the_entrance() {
    let resources = test_settlement_resources();
    for layout in layouts(&mut StdRng::seed_from_u64(7), resources.field_shapes()) {
      assert!(layout.tiles.contains(&(layout.entrance, TileShape::Fill)));
      assert_eq!(layout.entrance.x.abs() + layout.entrance.y.abs(), 1);
    }
  }

  #[test]
  fn field_type_maps_every_tile_shape_to_its_own_artwork() {
    for tile_shape in [
      TileShape::Fill,
      TileShape::EdgeTop,
      TileShape::EdgeRight,
      TileShape::EdgeBottom,
      TileShape::EdgeLeft,
      TileShape::OuterCornerTopLeft,
      TileShape::OuterCornerTopRight,
      TileShape::OuterCornerBottomRight,
      TileShape::OuterCornerBottomLeft,
      TileShape::InnerCornerTopLeft,
      TileShape::InnerCornerTopRight,
      TileShape::InnerCornerBottomRight,
      TileShape::InnerCornerBottomLeft,
    ] {
      assert!(FieldType::Wheat.object_name(tile_shape).is_wheat_field());
      assert!(FieldType::Pasture.object_name(tile_shape).is_pasture());
    }
  }

  #[test]
  fn permitted_field_types_follow_housing_capacity() {
    let mut rng = StdRng::seed_from_u64(7);

    assert!(permitted_field_types(&mut rng, 0).is_empty());
    assert_eq!(permitted_field_types(&mut rng, 1).len(), 1);
    assert_eq!(permitted_field_types(&mut rng, 3), vec![FieldType::Wheat, FieldType::Pasture]);
  }

  #[test]
  fn classify_tile_can_classify_every_field_shape() {
    let resources = test_settlement_resources();
    for shape in resources.field_shapes() {
      for &(x, y) in shape.tiles() {
        classify_tile(shape.tiles(), x, y);
      }
    }
  }

  #[test]
  fn classify_tile_maps_fill_edges_and_outer_corners() {
    let shape = rectangle(4, 4);
    assert_eq!(classify_tile(&shape, 1, 1), TileShape::Fill);
    assert_eq!(classify_tile(&shape, 1, 0), TileShape::EdgeTop);
    assert_eq!(classify_tile(&shape, 3, 1), TileShape::EdgeRight);
    assert_eq!(classify_tile(&shape, 1, 3), TileShape::EdgeBottom);
    assert_eq!(classify_tile(&shape, 0, 1), TileShape::EdgeLeft);
    assert_eq!(classify_tile(&shape, 0, 0), TileShape::OuterCornerTopLeft);
    assert_eq!(classify_tile(&shape, 3, 0), TileShape::OuterCornerTopRight);
    assert_eq!(classify_tile(&shape, 3, 3), TileShape::OuterCornerBottomRight);
    assert_eq!(classify_tile(&shape, 0, 3), TileShape::OuterCornerBottomLeft);
  }

  #[test]
  fn classify_tile_maps_each_missing_diagonal_to_an_inner_corner() {
    let cases = [
      ((0, 0), TileShape::InnerCornerTopLeft),
      ((2, 0), TileShape::InnerCornerTopRight),
      ((2, 2), TileShape::InnerCornerBottomRight),
      ((0, 2), TileShape::InnerCornerBottomLeft),
    ];
    for (missing, expected) in cases {
      let mut shape = rectangle(3, 3);
      shape.retain(|point| *point != missing);
      assert_eq!(classify_tile(&shape, 1, 1), expected);
    }
  }
}
