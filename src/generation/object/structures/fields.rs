use super::templates;
use crate::coordinates::Point;
use crate::coordinates::point::InternalGrid;
use crate::generation::object::lib::{ObjectGrid, ObjectName};
use crate::generation::object::structures::structure_generation::{
  select_fitting_building, update_path_in_front_of_entrance,
};
use bevy::log::*;
use bevy::platform::collections::HashSet;
use rand::prelude::StdRng;
use rand::seq::SliceRandom;
use rand::{RngExt, SeedableRng};

type FieldType = fn(u8) -> ObjectName;

/// Tile offsets relative to the road connection, and the entrance offset.
pub(super) struct FieldLayout {
  pub tiles: Vec<(Point<InternalGrid>, ObjectName)>,
  pub entrance: Point<InternalGrid>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
enum FarmTile {
  Fill,
  SideTop,
  SideRight,
  SideBottom,
  SideLeft,
  OuterCornerTopLeft,
  OuterCornerTopRight,
  OuterCornerBottomRight,
  OuterCornerBottomLeft,
  InnerCornerTopLeft,
  InnerCornerTopRight,
  InnerCornerBottomRight,
  InnerCornerBottomLeft,
}

const SIDES: [(i32, i32); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];
const DIAGONALS: [(i32, i32); 4] = [(-1, -1), (1, -1), (1, 1), (-1, 1)];

pub(super) fn place_fields(
  grid: &mut ObjectGrid,
  path_points: &[Point<InternalGrid>],
  available_grid_space: &HashSet<Point<InternalGrid>>,
  building_templates: &[templates::BuildingTemplate],
  occupied_grid_space: &mut HashSet<Point<InternalGrid>>,
  rng: &mut StdRng,
) -> i8 {
  let mut fields_placed = 0;
  let capacity = estimated_housing_capacity(path_points, available_grid_space, HashSet::new(), building_templates);
  let permitted_field_types: Vec<FieldType> = match permitted_field_types(rng, capacity) {
    Some(value) => value,
    None => return fields_placed,
  };
  let mut candidates = path_points.to_vec();
  candidates.shuffle(rng);
  for field_type in permitted_field_types {
    let layouts = layouts(field_type, rng);
    // Try the preferred footprint at all road sites before falling back to a smaller one
    'placement_loop: for layout in layouts {
      for &path_ig in &candidates {
        let tiles: Vec<(Point<InternalGrid>, ObjectName)> = layout
          .tiles
          .iter()
          .map(|(offset, name)| (Point::new_internal_grid(path_ig.x + offset.x, path_ig.y + offset.y), *name))
          .collect();
        if !tiles
          .iter()
          .all(|(point, _)| available_grid_space.contains(point) && !occupied_grid_space.contains(point))
        {
          continue;
        }
        // Do not bridge elevation changes with a single field
        let terrain = grid.get_cell(&tiles[0].0).map(|cell| cell.terrain());
        if !tiles
          .iter()
          .all(|(point, _)| grid.get_cell(point).map(|cell| cell.terrain()) == terrain)
        {
          continue;
        }

        // A settlement must retain room for housing, even on a cramped site
        let mut reserved_grid_space = occupied_grid_space.clone();
        reserved_grid_space.extend(tiles.iter().map(|(point, _)| *point));
        if estimated_housing_capacity(path_points, available_grid_space, reserved_grid_space, building_templates) == 0 {
          continue;
        }

        for (point, name) in &tiles {
          if let Some(cell) = grid.get_cell_mut(point) {
            cell.mark_as_collapsed(*name);
          }
        }
        occupied_grid_space.extend(tiles.iter().map(|(point, _)| *point));
        trace!(
          "Placed [{:?}] field with [{}] tiles beside {} on {}",
          field_type(0),
          tiles.len(),
          path_ig,
          grid.cg,
        );
        let entrance_ig = Point::new_internal_grid(path_ig.x + layout.entrance.x, path_ig.y + layout.entrance.y);
        update_path_in_front_of_entrance(&path_ig, &entrance_ig, grid);
        fields_placed += 1;
        break 'placement_loop;
      }
    }
  }
  fields_placed
}

/// Metadata only records settled/unsettled. Estimate local size from non-overlapping house sites, rather than road
/// length (which also includes wilderness roads).
fn estimated_housing_capacity(
  path_points: &[Point<InternalGrid>],
  available_grid_space: &HashSet<Point<InternalGrid>>,
  mut reserved_grid_space: HashSet<Point<InternalGrid>>,
  building_templates: &[templates::BuildingTemplate],
) -> usize {
  let mut rng = StdRng::seed_from_u64(0);
  let mut estimated_building_count = 0;
  for &ig in path_points {
    if let Some(template) =
      select_fitting_building(building_templates, ig, available_grid_space, &reserved_grid_space, &mut rng)
    {
      let origin = template.calculate_origin_ig_from_connection_point(ig);
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

/// Returns either `None`, a single random [`FieldType`], or both types - depending on whether there's enough capacity.   
fn permitted_field_types(rng: &mut StdRng, capacity: usize) -> Option<Vec<FieldType>> {
  let field_type = if rng.random_bool(0.5) {
    ObjectName::WheatField
  } else {
    ObjectName::Pasture
  };
  Some(match capacity {
    0 => return None,
    0..=2 => vec![field_type],
    _ => vec![ObjectName::WheatField, ObjectName::Pasture],
  })
}

pub(super) fn layouts(field: fn(u8) -> ObjectName, rng: &mut StdRng) -> Vec<FieldLayout> {
  let mut shapes = vec![
    rectangle(4, 3),
    rectangle(5, 3),
    rectangle(4, 4),
    rectangle(6, 3),
    rectangle(5, 4),
    rectangle(6, 4),
    l_shape(5, 4, 2, 2),
    l_shape(6, 5, 3, 2),
  ];
  shapes.shuffle(rng);

  let mut layouts = Vec::new();
  for mut shape in shapes {
    // Rotate footprints, not sprites
    for _ in 0..rand::RngExt::random_range(rng, 0..4) {
      shape = shape.into_iter().map(|(x, y)| (-y, x)).collect();
    }

    let mut entrances: Vec<((i32, i32), usize)> = shape
      .iter()
      .filter_map(|&(x, y)| {
        let outside: Vec<_> = SIDES
          .iter()
          .enumerate()
          .filter(|(_, (dx, dy))| !shape.contains(&(x + dx, y + dy)))
          .collect();
        (outside.len() == 1).then(|| ((x, y), outside[0].0))
      })
      .collect();
    entrances.shuffle(rng);

    for (entrance, entrance_side) in entrances {
      let (dx, dy) = SIDES[entrance_side];
      let tiles = shape
        .iter()
        .map(|&(x, y)| {
          let tile = if (x, y) == entrance {
            // The fill tile leaves a one-tile opening in the boundary fence
            FarmTile::Fill
          } else {
            classify_tile(&shape, x, y)
          };
          (
            Point::new_internal_grid(x - entrance.0 - dx, y - entrance.1 - dy),
            field(tile as u8),
          )
        })
        .collect();
      layouts.push(FieldLayout {
        tiles,
        entrance: Point::new_internal_grid(-dx, -dy),
      });
    }
  }
  layouts
}

fn classify_tile(shape: &[(i32, i32)], x: i32, y: i32) -> FarmTile {
  let outside: Vec<_> = SIDES
    .iter()
    .enumerate()
    .filter_map(|(side, (dx, dy))| (!shape.contains(&(x + dx, y + dy))).then_some(side))
    .collect();

  match outside.as_slice() {
    [] => {
      let missing_diagonal = DIAGONALS.iter().position(|(dx, dy)| !shape.contains(&(x + dx, y + dy)));
      match missing_diagonal {
        Some(0) => FarmTile::InnerCornerTopLeft,
        Some(1) => FarmTile::InnerCornerTopRight,
        Some(2) => FarmTile::InnerCornerBottomRight,
        Some(3) => FarmTile::InnerCornerBottomLeft,
        None => FarmTile::Fill,
        Some(_) => unreachable!(),
      }
    }
    [0] => FarmTile::SideTop,
    [1] => FarmTile::SideRight,
    [2] => FarmTile::SideBottom,
    [3] => FarmTile::SideLeft,
    [0, 1] => FarmTile::OuterCornerTopRight,
    [0, 3] => FarmTile::OuterCornerTopLeft,
    [1, 2] => FarmTile::OuterCornerBottomRight,
    [2, 3] => FarmTile::OuterCornerBottomLeft,
    _ => panic!("Farm shape has an unsupported one-tile-wide section at ({x}, {y})"),
  }
}

fn rectangle(width: i32, height: i32) -> Vec<(i32, i32)> {
  (0..height).flat_map(|y| (0..width).map(move |x| (x, y))).collect()
}

fn l_shape(width: i32, height: i32, vertical_width: i32, horizontal_height: i32) -> Vec<(i32, i32)> {
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
  fn layouts_use_only_the_thirteen_modular_tiles() {
    let layouts = layouts(ObjectName::WheatField, &mut StdRng::seed_from_u64(7));
    let tile_indices: Vec<_> = layouts
      .iter()
      .flat_map(|layout| layout.tiles.iter())
      .map(|(_, name)| match name {
        ObjectName::WheatField(tile) => *tile,
        _ => unreachable!(),
      })
      .collect();

    assert!(tile_indices.iter().all(|&tile| tile < 13));
  }

  #[test]
  fn classify_tile_maps_fill_sides_and_outer_corners() {
    let shape = rectangle(4, 4);
    assert_eq!(classify_tile(&shape, 1, 1), FarmTile::Fill);
    assert_eq!(classify_tile(&shape, 1, 0), FarmTile::SideTop);
    assert_eq!(classify_tile(&shape, 3, 1), FarmTile::SideRight);
    assert_eq!(classify_tile(&shape, 1, 3), FarmTile::SideBottom);
    assert_eq!(classify_tile(&shape, 0, 1), FarmTile::SideLeft);
    assert_eq!(classify_tile(&shape, 0, 0), FarmTile::OuterCornerTopLeft);
    assert_eq!(classify_tile(&shape, 3, 0), FarmTile::OuterCornerTopRight);
    assert_eq!(classify_tile(&shape, 3, 3), FarmTile::OuterCornerBottomRight);
    assert_eq!(classify_tile(&shape, 0, 3), FarmTile::OuterCornerBottomLeft);
  }

  #[test]
  fn classify_tile_maps_each_missing_diagonal_to_an_inner_corner() {
    let cases = [
      ((0, 0), FarmTile::InnerCornerTopLeft),
      ((2, 0), FarmTile::InnerCornerTopRight),
      ((2, 2), FarmTile::InnerCornerBottomRight),
      ((0, 2), FarmTile::InnerCornerBottomLeft),
    ];
    for (missing, expected) in cases {
      let mut shape = rectangle(3, 3);
      shape.retain(|point| *point != missing);
      assert_eq!(classify_tile(&shape, 1, 1), expected);
    }
  }
}
