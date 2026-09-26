use super::{buildings, fields};
use crate::constants::CHUNK_SIZE;
use crate::coordinates::point::{ChunkGrid, InternalGrid};
use crate::coordinates::{Direction, Point};
use crate::generation::model::{Metadata, SettlementResources};
use crate::generation::object::model::{BuildingTemplate, Cell, ObjectGrid, ObjectName};
use crate::generation::shared;
use crate::settings::Settings;
use bevy::app::{App, Plugin};
use bevy::log::*;
use bevy::platform::collections::HashSet;
use rand::RngExt;
use rand::prelude::StdRng;

/// Contains the main logic for generation of settlement structure - such as buildings and fields - in the world. This
/// happens after path generation and prior to generating other decorative objects.
pub struct SettlementGenerationPlugin;

impl Plugin for SettlementGenerationPlugin {
  fn build(&self, _app: &mut App) {}
}

/// The entry point for generating a settlement and placing it on the [`ObjectGrid`].
pub fn place_settlement_on_grid(
  object_grid: &mut ObjectGrid,
  settings: &Settings,
  metadata: &Metadata,
  settlement_resources: &SettlementResources,
  rng: &mut StdRng,
) {
  let start_time = shared::get_time();
  let cg = object_grid.cg;
  if !settings.object.generate_paths || !settings.object.generate_settlements {
    debug!(
      "Skipped generating settlements for {} because it or path generation are disabled",
      cg
    );
    return;
  }
  if !metadata.get_settlement_status_for(&cg) {
    debug!(
      "Skipped generating settlements for {} because it is not marked as settled in metadata",
      cg
    );
    return;
  }

  // Determine path points along which we can generate settlement structures
  let mut path_points: Vec<Point<InternalGrid>> = vec![];
  add_valid_connection_points(&mut path_points, object_grid, metadata, &cg);
  add_points_from_generated_path(&mut path_points, object_grid, settings, rng, &cg);
  clean_up_path_points(&mut path_points, object_grid);
  if path_points.is_empty() {
    debug!(
      "Skipped generating settlements because there are no valid path points for {}",
      cg
    );
    return;
  }

  // Now place settlement structures on the grid
  let available_grid_space = compute_available_space_map(object_grid);
  let building_templates = settlement_resources.building_templates();
  let mut occupied_grid_space = HashSet::new();
  let fields_placed = fields::place_fields(
    object_grid,
    &path_points,
    &available_grid_space,
    building_templates,
    settlement_resources.field_shapes(),
    &mut occupied_grid_space,
    rng,
  );
  let buildings_placed = buildings::place_buildings(
    object_grid,
    &path_points,
    &available_grid_space,
    building_templates,
    &mut occupied_grid_space,
    rng,
    cg,
  );

  debug!(
    "Placed [{}] buildings(s) and [{}] field(s) on grid for {} in {} ms on {}",
    buildings_placed,
    fields_placed,
    object_grid.cg,
    shared::get_time() - start_time,
    shared::thread_name()
  );
}

/// Fetches the connection points for this chunk from the [`Metadata`] and appends them to the list of proposed points.
fn add_valid_connection_points(
  proposed_points: &mut Vec<Point<InternalGrid>>,
  object_grid: &mut ObjectGrid,
  metadata: &Metadata,
  cg: &Point<ChunkGrid>,
) {
  let mut connection_points = metadata.get_connection_points_for(cg, object_grid);
  if connection_points.len() > 1 {
    connection_points = connection_points
      .iter_mut()
      .filter(|cp| !cp.is_touching_edge())
      .map(|cp| *cp)
      .collect::<Vec<Point<InternalGrid>>>();
  }
  if connection_points.is_empty() {
    trace!("No valid connection points found for {}", cg);
  }
  proposed_points.append(&mut connection_points);
}

/// Appends points along the previously calculated path to the list of proposed path points. The density settings for
/// structures determines the likelihood of a point being sampled out.
fn add_points_from_generated_path(
  path_points: &mut Vec<Point<InternalGrid>>,
  object_grid: &mut ObjectGrid,
  settings: &Settings,
  rng: &mut StdRng,
  cg: &Point<ChunkGrid>,
) {
  let structure_density = settings.object.settlement_density;
  let mut points_from_generated_path_to_add: Vec<Point<InternalGrid>> =
    object_grid.get_generated_path().iter().copied().collect();
  points_from_generated_path_to_add.sort_by_key(|point| (point.y, point.x));
  points_from_generated_path_to_add.retain(|_| rng.random_range(0.0..1.0) <= structure_density);
  trace!(
    "Adding [{}/{}] path points from the generated path for {} based on settlement structure density of [{:.2}]",
    points_from_generated_path_to_add.len(),
    object_grid.get_generated_path().len(),
    cg,
    structure_density
  );
  path_points.append(&mut points_from_generated_path_to_add);
}

/// Metadata may include endpoints for paths that could not be generated, so they are removed here.
fn clean_up_path_points(path_points: &mut Vec<Point<InternalGrid>>, object_grid: &mut ObjectGrid) {
  path_points.sort_by_key(|point| (point.y, point.x));
  path_points.dedup();
  path_points.retain(|point| {
    object_grid.get_cell(point).is_some_and(|cell| {
      cell.is_collapsed() && cell.get_possible_states().first().is_some_and(|state| state.name.is_path())
    })
  });
}

/// Returns a map of space available for placing buildings.
pub fn compute_available_space_map(object_grid: &mut ObjectGrid) -> HashSet<Point<InternalGrid>> {
  let mut available_space = HashSet::new();
  for y in 0..CHUNK_SIZE {
    for x in 0..CHUNK_SIZE {
      let ig = Point::new_internal_grid(x, y);
      if let Some(cell) = object_grid.get_cell_mut(&ig)
        && !cell.is_collapsed()
        && cell.is_suitable_for_building_placement()
      {
        available_space.insert(ig);
      }
    }
  }

  available_space
}

/// Selects a building template that fits at the given path connection point without overlapping any occupied space.
/// If multiple templates fit, one is chosen at random. If none fit, `None` is returned.
pub fn select_fitting_building(
  building_templates: &[BuildingTemplate],
  path_connection_ig: Point<InternalGrid>,
  available_space: &HashSet<Point<InternalGrid>>,
  occupied_space: &HashSet<Point<InternalGrid>>,
  rng: &mut StdRng,
) -> Option<BuildingTemplate> {
  let mut fitting_building_templates = Vec::new();
  for template in building_templates {
    if template.is_placeable_at_path(path_connection_ig, available_space) {
      let origin_ig = template.calculate_origin_ig_from_connection_point(path_connection_ig);
      let mut is_overlapping = false;
      for y in 0..template.height {
        for x in 0..template.width {
          let ig = Point::new_internal_grid(origin_ig.x + x, origin_ig.y + y);
          if occupied_space.contains(&ig) {
            is_overlapping = true;
            break;
          }
        }
        if is_overlapping {
          break;
        }
      }
      if !is_overlapping {
        fitting_building_templates.push(template.clone());
      }
    }
  }
  if fitting_building_templates.is_empty() {
    return None;
  }
  let index = rng.random_range(0..fitting_building_templates.len());

  Some(fitting_building_templates[index].clone())
}

/// Updates the object name of the cell at the given connection point to ensure the path connects to the entrance
/// correctly. Without this, there would be a gap in the path leading to the entrance.
pub fn update_path_in_front_of_entrance(
  path_connection_ig: &Point<InternalGrid>,
  absolute_entrance_ig: &Point<InternalGrid>,
  object_grid: &mut ObjectGrid,
) {
  let cg = object_grid.cg;
  if let Some(cell) = object_grid.get_cell_mut(path_connection_ig) {
    let object_name = determine_updated_object_name(cell, path_connection_ig, absolute_entrance_ig, &cg);
    cell.mark_as_collapsed(object_name);
  } else {
    error!(
      "Failed to get cell at connection point {:?} on {} to update path in front of entrance at {:?}",
      path_connection_ig, cg, absolute_entrance_ig
    );
  }
}

/// Returns the updated object name for a path object at the given connection point based on the location of and
/// direction to the entrance.
fn determine_updated_object_name(
  cell: &mut Cell,
  path_connection_ig: &Point<InternalGrid>,
  absolute_entrance_ig: &Point<InternalGrid>,
  cg: &Point<ChunkGrid>,
) -> ObjectName {
  let terrain_states = cell.get_possible_states();
  if !cell.is_collapsed() || terrain_states.len() != 1 {
    error!(
      "Expected collapsed path tile at connection point {:?} on {} but found: {:?} ",
      path_connection_ig, cg, cell
    );
    return ObjectName::PathUndefined;
  }
  let terrain_state = &terrain_states[0];
  if !terrain_state.name.is_path() {
    error!(
      "Expected path tile at connection point {:?} on {} but found: {:?}",
      path_connection_ig, cg, cell
    );
    return ObjectName::PathUndefined;
  }

  let missing_direction = Direction::from_points(path_connection_ig, absolute_entrance_ig);
  let new_object_name = match (terrain_state.name, missing_direction) {
    (ObjectName::PathTop, Direction::Left) => ObjectName::PathTopLeft,
    (ObjectName::PathTop, Direction::Right) => ObjectName::PathTopRight,
    (ObjectName::PathTop, Direction::Bottom) => ObjectName::PathVertical,
    (ObjectName::PathRight, Direction::Top) => ObjectName::PathTopRight,
    (ObjectName::PathRight, Direction::Bottom) => ObjectName::PathBottomRight,
    (ObjectName::PathRight, Direction::Left) => ObjectName::PathHorizontal,
    (ObjectName::PathBottom, Direction::Left) => ObjectName::PathBottomLeft,
    (ObjectName::PathBottom, Direction::Right) => ObjectName::PathBottomRight,
    (ObjectName::PathBottom, Direction::Top) => ObjectName::PathVertical,
    (ObjectName::PathTopRight, Direction::Bottom) => ObjectName::PathRightVertical,
    (ObjectName::PathTopRight, Direction::Left) => ObjectName::PathTopHorizontal,
    (ObjectName::PathTopLeft, Direction::Bottom) => ObjectName::PathLeftVertical,
    (ObjectName::PathTopLeft, Direction::Right) => ObjectName::PathTopHorizontal,
    (ObjectName::PathBottomLeft, Direction::Top) => ObjectName::PathLeftVertical,
    (ObjectName::PathBottomLeft, Direction::Right) => ObjectName::PathBottomHorizontal,
    (ObjectName::PathBottomRight, Direction::Top) => ObjectName::PathRightVertical,
    (ObjectName::PathBottomRight, Direction::Left) => ObjectName::PathBottomHorizontal,
    (ObjectName::PathLeft, Direction::Top) => ObjectName::PathTopLeft,
    (ObjectName::PathLeft, Direction::Bottom) => ObjectName::PathBottomLeft,
    (ObjectName::PathLeft, Direction::Right) => ObjectName::PathHorizontal,
    (ObjectName::PathVertical, Direction::Left) => ObjectName::PathLeftVertical,
    (ObjectName::PathVertical, Direction::Right) => ObjectName::PathRightVertical,
    (ObjectName::PathHorizontal, Direction::Top) => ObjectName::PathTopHorizontal,
    (ObjectName::PathHorizontal, Direction::Bottom) => ObjectName::PathBottomHorizontal,
    (ObjectName::PathTopHorizontal, Direction::Bottom) => ObjectName::PathCross,
    (ObjectName::PathBottomHorizontal, Direction::Top) => ObjectName::PathCross,
    (ObjectName::PathLeftVertical, Direction::Right) => ObjectName::PathCross,
    (ObjectName::PathRightVertical, Direction::Left) => ObjectName::PathCross,
    // Connecting an already-connected side is valid and must not add another branch:
    (ObjectName::PathTop, Direction::Top)
    | (ObjectName::PathRight, Direction::Right)
    | (ObjectName::PathBottom, Direction::Bottom)
    | (ObjectName::PathLeft, Direction::Left)
    | (ObjectName::PathTopRight, Direction::Top | Direction::Right)
    | (ObjectName::PathTopLeft, Direction::Top | Direction::Left)
    | (ObjectName::PathBottomRight, Direction::Bottom | Direction::Right)
    | (ObjectName::PathBottomLeft, Direction::Bottom | Direction::Left)
    | (ObjectName::PathVertical, Direction::Top | Direction::Bottom)
    | (ObjectName::PathHorizontal, Direction::Left | Direction::Right)
    | (ObjectName::PathTopHorizontal, Direction::Top | Direction::Left | Direction::Right)
    | (ObjectName::PathBottomHorizontal, Direction::Bottom | Direction::Left | Direction::Right)
    | (ObjectName::PathLeftVertical, Direction::Top | Direction::Bottom | Direction::Left)
    | (ObjectName::PathRightVertical, Direction::Top | Direction::Bottom | Direction::Right)
    | (ObjectName::PathCross, Direction::Top | Direction::Right | Direction::Bottom | Direction::Left) => terrain_state.name,
    _ => panic!(
      "Unsupported road connection: [{:?}] at {:?} on {} towards [{:?}], entrance at {}",
      terrain_state.name, path_connection_ig, cg, missing_direction, absolute_entrance_ig
    ),
  };

  trace!(
    "Updated object name of cell {} on {} from [{:?}] to [{:?}] because a building with a entrance at {} was placed",
    path_connection_ig, cg, terrain_state.name, new_object_name, absolute_entrance_ig
  );

  new_object_name
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::generation::model::{TerrainType, TileType};
  use crate::generation::object::settlements::test_settlement_resources;
  use rand::SeedableRng;

  fn test_settlement(width: i32) -> (ObjectGrid, Settings, Metadata) {
    let cg = Point::new_chunk_grid(0, 0);
    let mut grid = ObjectGrid::default(cg);
    for y in 2..10 {
      for x in 1..width {
        let cell = grid.get_cell_mut(&Point::new_internal_grid(x, y)).unwrap();
        cell.initialise(
          TerrainType::Land2,
          TileType::Fill,
          &[],
          vec![(TerrainType::Land1, TileType::Fill)],
          false,
        );
      }
    }
    let path: HashSet<_> = (1..width).map(|x| Point::new_internal_grid(x, 6)).collect();
    for point in &path {
      grid
        .get_cell_mut(point)
        .unwrap()
        .mark_as_collapsed(ObjectName::PathHorizontal);
    }
    grid.set_generated_path(path);
    let mut settings = Settings::default();
    settings.object.settlement_density = 1.0;
    let mut metadata = Metadata::default(cg);
    metadata.settlement.insert(cg, true);
    metadata.connection.insert(cg, vec![]);
    (grid, settings, metadata)
  }

  fn object_names(grid: &ObjectGrid) -> Vec<ObjectName> {
    (0..CHUNK_SIZE)
      .flat_map(|y| (0..CHUNK_SIZE).map(move |x| Point::new_internal_grid(x, y)))
      .filter_map(|point| {
        grid
          .get_cell(&point)
          .unwrap()
          .get_possible_states()
          .first()
          .map(|state| state.name)
      })
      .collect()
  }

  fn place_test_settlement(grid: &mut ObjectGrid, settings: &Settings, metadata: &Metadata, rng: &mut StdRng) {
    let resources = test_settlement_resources();
    place_settlement_on_grid(grid, settings, metadata, &resources, rng);
  }

  #[test]
  #[should_panic(expected = "Unsupported road connection")]
  fn road_connection_rejects_an_entrance_on_the_road_cell() {
    let point = Point::new_internal_grid(5, 5);
    let mut cell = Cell::new(5, 5);
    cell.mark_as_collapsed(ObjectName::PathCross);
    determine_updated_object_name(&mut cell, &point, &point, &Point::new_chunk_grid(0, 0));
  }

  #[test]
  #[should_panic(expected = "Unsupported road connection")]
  fn road_connection_rejects_a_diagonal_entrance() {
    let mut cell = Cell::new(5, 5);
    cell.mark_as_collapsed(ObjectName::PathCross);
    determine_updated_object_name(
      &mut cell,
      &Point::new_internal_grid(5, 5),
      &Point::new_internal_grid(6, 4),
      &Point::new_chunk_grid(0, 0),
    );
  }

  #[test]
  #[should_panic(expected = "Unsupported road connection")]
  fn road_connection_rejects_an_undefined_path() {
    let mut cell = Cell::new(5, 5);
    cell.mark_as_collapsed(ObjectName::PathUndefined);
    determine_updated_object_name(
      &mut cell,
      &Point::new_internal_grid(5, 5),
      &Point::new_internal_grid(5, 4),
      &Point::new_chunk_grid(0, 0),
    );
  }

  #[test]
  fn road_connection_adds_only_the_requested_cardinal_branch() {
    // Indices encode Top=1, Right=2, Bottom=4, Left=8.
    let paths = [
      ObjectName::PathUndefined,
      ObjectName::PathTop,
      ObjectName::PathRight,
      ObjectName::PathTopRight,
      ObjectName::PathBottom,
      ObjectName::PathVertical,
      ObjectName::PathBottomRight,
      ObjectName::PathRightVertical,
      ObjectName::PathLeft,
      ObjectName::PathTopLeft,
      ObjectName::PathHorizontal,
      ObjectName::PathTopHorizontal,
      ObjectName::PathBottomLeft,
      ObjectName::PathLeftVertical,
      ObjectName::PathBottomHorizontal,
      ObjectName::PathCross,
    ];
    let point = Point::new_internal_grid(5, 5);
    for mask in 1..paths.len() {
      for (bit, dx, dy) in [(1, 0, -1), (2, 1, 0), (4, 0, 1), (8, -1, 0)] {
        let mut cell = Cell::new(5, 5);
        cell.mark_as_collapsed(paths[mask]);
        let entrance = Point::new_internal_grid(point.x + dx, point.y + dy);
        assert_eq!(
          determine_updated_object_name(&mut cell, &point, &entrance, &Point::new_chunk_grid(0, 0)),
          paths[mask | bit],
          "Road {:?}, direction ({dx}, {dy})",
          paths[mask]
        );
      }
    }
  }

  #[test]
  fn connecting_an_already_connected_road_does_not_add_spurs() {
    let point = Point::new_internal_grid(5, 5);
    let entrance = Point::new_internal_grid(5, 4);
    let cg = Point::new_chunk_grid(0, 0);
    for name in [ObjectName::PathTop, ObjectName::PathTopHorizontal, ObjectName::PathCross] {
      let mut cell = Cell::new(5, 5);
      cell.mark_as_collapsed(name);
      assert_eq!(determine_updated_object_name(&mut cell, &point, &entrance, &cg), name);
    }
  }

  #[test]
  fn place_settlement_on_grid_small_settlement_includes_a_field() {
    let (mut grid, settings, metadata) = test_settlement(8);
    place_test_settlement(&mut grid, &settings, &metadata, &mut StdRng::seed_from_u64(7));
    let field_tiles = object_names(&grid).iter().filter(|name| name.is_field()).count();
    assert!((12..=24).contains(&field_tiles));
  }

  #[test]
  fn place_settlement_on_grid_respects_generation_switches_and_unsettled_chunks() {
    for disabled in 0..3 {
      let (mut grid, mut settings, mut metadata) = test_settlement(16);
      match disabled {
        0 => settings.object.generate_settlements = false,
        1 => settings.object.generate_paths = false,
        _ => {
          metadata.settlement.insert(grid.cg, false);
        }
      }
      let before = object_names(&grid);
      place_test_settlement(&mut grid, &settings, &metadata, &mut StdRng::seed_from_u64(7));
      assert_eq!(object_names(&grid), before);
    }
  }

  #[test]
  fn place_settlement_on_grid_is_repeatable_despite_path_insertion_order() {
    let (mut first, settings, metadata) = test_settlement(16);
    let (mut second, _, _) = test_settlement(16);
    second.set_generated_path((1..16).rev().map(|x| Point::new_internal_grid(x, 6)).collect());
    place_test_settlement(&mut first, &settings, &metadata, &mut StdRng::seed_from_u64(42));
    place_test_settlement(&mut second, &settings, &metadata, &mut StdRng::seed_from_u64(42));
    assert_eq!(object_names(&first), object_names(&second));
  }

  #[test]
  fn place_settlement_on_grid_preserves_existing_objects_with_fields() {
    for seed in 0..20 {
      let (mut grid, settings, metadata) = test_settlement(16);
      let obstacle = Point::new_internal_grid(7, 4);
      grid
        .get_cell_mut(&obstacle)
        .unwrap()
        .mark_as_collapsed(ObjectName::HouseSmallWallLeft1);
      place_test_settlement(&mut grid, &settings, &metadata, &mut StdRng::seed_from_u64(seed));
      assert_eq!(
        grid.get_cell(&obstacle).unwrap().get_possible_states()[0].name,
        ObjectName::HouseSmallWallLeft1
      );
      let names = object_names(&grid);
      assert!(names.iter().any(ObjectName::is_wheat_field));
      assert!(names.iter().any(ObjectName::is_pasture));
    }
  }

  #[test]
  fn place_settlement_on_grid_cramped_site_never_places_partial_fields() {
    let (mut grid, settings, metadata) = test_settlement(3);
    place_test_settlement(&mut grid, &settings, &metadata, &mut StdRng::seed_from_u64(7));
    assert!(!object_names(&grid).iter().any(ObjectName::is_field));
  }

  #[test]
  fn place_settlement_on_grid_medium_settlement_has_wheat_fields_and_pastures() {
    let (mut grid, settings, metadata) = test_settlement(11);
    place_test_settlement(&mut grid, &settings, &metadata, &mut StdRng::seed_from_u64(7));
    let names = object_names(&grid);
    assert!(names.iter().any(ObjectName::is_wheat_field));
    assert!(names.iter().any(ObjectName::is_pasture));
  }

  #[test]
  fn place_settlement_on_grid_large_settlement_has_fields_and_buildings() {
    let (mut grid, settings, metadata) = test_settlement(16);
    place_test_settlement(&mut grid, &settings, &metadata, &mut StdRng::seed_from_u64(7));
    let names = object_names(&grid);
    assert!(names.iter().any(ObjectName::is_wheat_field));
    assert!(names.iter().any(ObjectName::is_pasture));
    assert!(names.iter().any(ObjectName::is_building));
    for point in grid.get_generated_path() {
      assert!(grid.get_cell(point).unwrap().get_possible_states()[0].name.is_path());
    }
  }

  #[test]
  fn fields_avoid_existing_objects_and_elevation_changes() {
    for seed in 0..20 {
      let (mut grid, settings, metadata) = test_settlement(16);
      let obstacle = Point::new_internal_grid(7, 4);
      grid
        .get_cell_mut(&obstacle)
        .unwrap()
        .mark_as_collapsed(ObjectName::HouseSmallWallLeft1);
      let raised = grid.get_cell_mut(&Point::new_internal_grid(8, 4)).unwrap();
      *raised = Cell::new(8, 4);
      raised.initialise(
        TerrainType::Land3,
        TileType::Fill,
        &[],
        vec![(TerrainType::Land1, TileType::Fill)],
        false,
      );
      place_test_settlement(&mut grid, &settings, &metadata, &mut StdRng::seed_from_u64(seed));
      assert_eq!(
        grid.get_cell(&obstacle).unwrap().get_possible_states()[0].name,
        ObjectName::HouseSmallWallLeft1
      );
      let mut field_count = 0;
      for y in 0..CHUNK_SIZE {
        for x in 0..CHUNK_SIZE {
          let cell = grid.get_cell(&Point::new_internal_grid(x, y)).unwrap();
          if cell.get_possible_states().first().is_some_and(|state| state.name.is_field()) {
            field_count += 1;
            assert_eq!(cell.terrain(), TerrainType::Land2);
          }
        }
      }
      assert!(field_count >= 12);
    }
  }

  #[test]
  fn fields_vary_in_size_and_are_fenced_beside_the_path() {
    let sides = [(0, -1), (1, 0), (0, 1), (-1, 0)];
    let mut areas = HashSet::new();
    let mut field_sides = HashSet::new();
    let mut has_non_rectangular = false;
    for seed in 0..80 {
      let (mut grid, settings, metadata) = test_settlement(12);
      place_test_settlement(&mut grid, &settings, &metadata, &mut StdRng::seed_from_u64(seed));
      let is_wheat_field = object_names(&grid)
        .into_iter()
        .find(ObjectName::is_field)
        .unwrap()
        .is_wheat_field();
      let cells: Vec<_> = (0..CHUNK_SIZE)
        .flat_map(|y| (0..CHUNK_SIZE).map(move |x| Point::new_internal_grid(x, y)))
        .filter_map(|point| {
          let name = grid.get_cell(&point)?.get_possible_states().first()?.name;
          ((is_wheat_field && name.is_wheat_field()) || (!is_wheat_field && name.is_pasture())).then_some((point, name))
        })
        .collect();
      assert!((12..=24).contains(&cells.len()));
      areas.insert(cells.len());
      let positions: HashSet<_> = cells.iter().map(|(point, _)| *point).collect();
      let width =
        positions.iter().map(|point| point.x).max().unwrap() - positions.iter().map(|point| point.x).min().unwrap() + 1;
      let height =
        positions.iter().map(|point| point.y).max().unwrap() - positions.iter().map(|point| point.y).min().unwrap() + 1;
      has_non_rectangular |= cells.len() < (width * height) as usize;
      let mut reached = HashSet::from([cells[0].0]);
      let mut frontier = vec![cells[0].0];
      while let Some(point) = frontier.pop() {
        for (dx, dy) in sides {
          let neighbour = Point::new_internal_grid(point.x + dx, point.y + dy);
          if positions.contains(&neighbour) && reached.insert(neighbour) {
            frontier.push(neighbour);
          }
        }
      }
      assert_eq!(reached, positions);

      let path_edges: Vec<_> = cells
        .iter()
        .flat_map(|(point, name)| {
          sides.iter().enumerate().filter_map(|(side, (dx, dy))| {
            let neighbour = Point::new_internal_grid(point.x + dx, point.y + dy);
            grid.get_generated_path().contains(&neighbour).then_some((side, *name))
          })
        })
        .collect();
      assert!(!path_edges.is_empty());
      assert!(
        path_edges
          .iter()
          .all(|(_, name)| !matches!(name, ObjectName::WheatFieldFill | ObjectName::PastureFill))
      );
      field_sides.extend(path_edges.into_iter().map(|(side, _)| side));
    }
    assert!(areas.len() >= 3, "Expected several field sizes, found {areas:?}");
    assert!(has_non_rectangular, "Expected an L-shaped field");
    assert!(field_sides.len() >= 2, "Expected fields on both sides of the road");
  }
}
