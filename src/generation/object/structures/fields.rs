use super::templates;
use crate::coordinates::Point;
use crate::coordinates::point::InternalGrid;
use crate::generation::lib::Direction;
use crate::generation::object::lib::{ObjectGrid, ObjectName};
use crate::generation::object::structures::structure_generation::{
  select_fitting_building, update_path_in_front_of_entrance,
};
use bevy::log::*;
use bevy::platform::collections::HashSet;
use rand::prelude::StdRng;
use rand::seq::SliceRandom;
use rand::{RngExt, SeedableRng};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FieldKind {
  Wheat,
  Pasture,
}

impl FieldKind {
  /// Maps tile geometry to artwork for this field kind.
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
  building_templates: &[templates::BuildingTemplate],
  occupied_grid_space: &mut HashSet<Point<InternalGrid>>,
  rng: &mut StdRng,
) -> i8 {
  let capacity = estimated_housing_capacity(
    path_points,
    available_grid_space,
    occupied_grid_space.clone(),
    building_templates,
  );
  let field_kinds = permitted_field_kinds(rng, capacity);
  let mut candidate_connection_igs = path_points.to_vec();
  candidate_connection_igs.shuffle(rng);
  let mut fields_placed = 0;

  for field_kind in field_kinds {
    // Try every shape, orientation, and entrance until one fits
    'placement_loop: for layout in layouts(rng) {
      for &connection_ig in &candidate_connection_igs {
        let proposed_field = absolute_tiles(&layout, connection_ig, field_kind);
        if !can_place_field(
          grid,
          &proposed_field,
          path_points,
          available_grid_space,
          occupied_grid_space,
          building_templates,
        ) {
          continue;
        }

        for (field_ig, name) in &proposed_field {
          if let Some(cell) = grid.get_cell_mut(field_ig) {
            cell.mark_as_collapsed(*name);
          }
        }
        occupied_grid_space.extend(proposed_field.iter().map(|(point, _)| *point));
        let entrance = Point::new_internal_grid(connection_ig.x + layout.entrance.x, connection_ig.y + layout.entrance.y);
        update_path_in_front_of_entrance(&connection_ig, &entrance, grid);
        fields_placed += 1;
        trace!(
          "Placed [{:?}] field with [{}] tiles beside {} on {}",
          field_kind,
          proposed_field.len(),
          connection_ig,
          grid.cg,
        );
        break 'placement_loop;
      }
    }
  }

  fields_placed
}

/// Anchors layout offsets to a road point and selects the field artwork.
fn absolute_tiles(
  layout: &FieldLayout,
  path_point: Point<InternalGrid>,
  field_kind: FieldKind,
) -> Vec<(Point<InternalGrid>, ObjectName)> {
  layout
    .tiles
    .iter()
    .map(|(offset, tile_shape)| {
      (
        Point::new_internal_grid(path_point.x + offset.x, path_point.y + offset.y),
        field_kind.object_name(*tile_shape),
      )
    })
    .collect()
}

/// Returns `false` for overlaps, elevation changes, and fields that take up too much space.
fn can_place_field(
  grid: &ObjectGrid,
  proposed_field_tiles: &[(Point<InternalGrid>, ObjectName)],
  path_points: &[Point<InternalGrid>],
  available_grid_space: &HashSet<Point<InternalGrid>>,
  occupied_grid_space: &HashSet<Point<InternalGrid>>,
  building_templates: &[templates::BuildingTemplate],
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
  estimated_housing_capacity(path_points, available_grid_space, reserved_grid_space, building_templates) > 0
}

/// Estimates settlement size from non-overlapping house sites because roads may extend into wilderness.
fn estimated_housing_capacity(
  path_points: &[Point<InternalGrid>],
  available_grid_space: &HashSet<Point<InternalGrid>>,
  mut reserved_grid_space: HashSet<Point<InternalGrid>>,
  building_templates: &[templates::BuildingTemplate],
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

/// Limits field variety according to the estimated settlement size.
fn permitted_field_kinds(rng: &mut StdRng, capacity: usize) -> Vec<FieldKind> {
  match capacity {
    0 => Vec::new(),
    1..=2 => vec![if rng.random_bool(0.5) {
      FieldKind::Wheat
    } else {
      FieldKind::Pasture
    }],
    _ => vec![FieldKind::Wheat, FieldKind::Pasture],
  }
}

/// Builds randomised candidates, trying one rotation per shape before fallback rotations.
fn layouts(rng: &mut StdRng) -> Vec<FieldLayout> {
  let mut shapes = field_shapes();
  shapes.shuffle(rng);
  let mut preferred_layouts = Vec::new();
  let mut fallback_layouts = Vec::new();
  for shape in shapes {
    let mut rotated_shapes = rotations(&shape);
    let first_rotation = rng.random_range(0..rotated_shapes.len());
    rotated_shapes.rotate_left(first_rotation);

    for (rotation_index, rotated_shape) in rotated_shapes.into_iter().enumerate() {
      let mut shape_entrances = entrances(&rotated_shape);
      if rotation_index == 0 {
        shape_entrances.shuffle(rng);
      }
      let target = if rotation_index == 0 {
        &mut preferred_layouts
      } else {
        &mut fallback_layouts
      };
      target.extend(
        shape_entrances
          .into_iter()
          .map(|(entrance, outside)| layout_from_entrance(&rotated_shape, entrance, outside)),
      );
    }
  }
  preferred_layouts.extend(fallback_layouts);
  preferred_layouts
}

/// Defines the permitted field sizes and proportions independently of placement.
fn field_shapes() -> Vec<Shape> {
  vec![
    rectangle(4, 3),
    rectangle(5, 3),
    rectangle(4, 4),
    rectangle(6, 3),
    rectangle(5, 4),
    rectangle(6, 4),
    l_shape(5, 4, 2, 2),
    l_shape(6, 5, 3, 2),
  ]
}

/// Produces every orientation so a constrained site does not fail on one random rotation.
fn rotations(shape: &[(i32, i32)]) -> Vec<Shape> {
  let mut rotations = Vec::with_capacity(4);
  let mut rotated = shape.to_vec();
  for _ in 0..4 {
    rotations.push(rotated.clone());
    rotated = rotated.into_iter().map(|(x, y)| (-y, x)).collect();
  }
  rotations
}

/// Finds boundary tiles with exactly one exposed side suitable for a road connection.
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

/// Anchors a shape beside the road and removes the fence at its chosen entrance.
fn layout_from_entrance(shape: &[(i32, i32)], entrance: (i32, i32), outside: Direction) -> FieldLayout {
  let outside_offset: Point<InternalGrid> = outside.to_point();
  let tiles = shape
    .iter()
    .map(|&(x, y)| {
      let tile_shape = if (x, y) == entrance {
        // Fill artwork leaves an opening in the boundary fence.
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

/// Selects edge and corner geometry from a tile's missing neighbours.
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

/// Distinguishes interior fill from concave corners using diagonal neighbours.
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

/// Lists cardinal sides with no adjacent field tile.
fn outside(shape: &[(i32, i32)], x: &i32, y: &i32) -> Vec<Direction> {
  CARDINAL_DIRECTIONS
    .into_iter()
    .filter(|direction| {
      let offset: Point<InternalGrid> = direction.to_point();
      !shape.contains(&(x + offset.x, y + offset.y))
    })
    .collect()
}

/// Builds a filled rectangle for the field-shape catalogue.
fn rectangle(width: i32, height: i32) -> Shape {
  (0..height).flat_map(|y| (0..width).map(move |x| (x, y))).collect()
}

/// Builds a concave field shape that exercises inner-corner artwork.
fn l_shape(width: i32, height: i32, vertical_width: i32, horizontal_height: i32) -> Shape {
  (0..height)
    .flat_map(|y| {
      (0..width)
        .filter(move |&x| x < vertical_width || y < horizontal_height)
        .map(move |x| (x, y))
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;
  use rand::SeedableRng;

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
    for layout in layouts(&mut StdRng::seed_from_u64(7)) {
      assert!(layout.tiles.contains(&(layout.entrance, TileShape::Fill)));
      assert_eq!(layout.entrance.x.abs() + layout.entrance.y.abs(), 1);
    }
  }

  #[test]
  fn field_kind_maps_every_tile_shape_to_its_own_artwork() {
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
      assert!(FieldKind::Wheat.object_name(tile_shape).is_wheat_field());
      assert!(FieldKind::Pasture.object_name(tile_shape).is_pasture());
    }
  }

  #[test]
  fn permitted_field_kinds_follow_housing_capacity() {
    let mut rng = StdRng::seed_from_u64(7);

    assert!(permitted_field_kinds(&mut rng, 0).is_empty());
    assert_eq!(permitted_field_kinds(&mut rng, 1).len(), 1);
    assert_eq!(permitted_field_kinds(&mut rng, 3), vec![FieldKind::Wheat, FieldKind::Pasture]);
  }

  #[test]
  fn classify_tile_can_classify_every_field_shape() {
    for shape in field_shapes() {
      for &(x, y) in &shape {
        classify_tile(&shape, x, y);
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
