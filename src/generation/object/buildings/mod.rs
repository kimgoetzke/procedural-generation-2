use crate::constants::CHUNK_SIZE;
use crate::coords::Point;
use crate::coords::point::{ChunkGrid, InternalGrid};
use crate::generation::lib::{Direction, shared};
use crate::generation::object::lib::{Cell, ObjectGrid, ObjectName};
use crate::generation::resources::Metadata;
use crate::resources::Settings;
use bevy::app::{App, Plugin};
use bevy::log::*;
use bevy::platform::collections::HashSet;
use rand::Rng;
use rand::prelude::StdRng;

/// Contains the main logic for generation of buildings in the world. This happens after path generation and prior to
/// generating other decorative objects.
pub struct BuildingGenerationPlugin;

impl Plugin for BuildingGenerationPlugin {
  fn build(&self, _app: &mut App) {}
}

#[derive(Debug, Clone)]
struct BuildingTemplate {
  name: String,
  width: i32,
  height: i32,
  tiles: Vec<Vec<ObjectName>>,
  /// The position of the door tile relative to the building's top-left corner in internal grid coordinates.  Remember
  /// that `x` is the column number and `y` is the row number of the door's position in the `tiles` 2D array.
  relative_door_ig: Point<InternalGrid>,
  connection_direction: Direction,
}

impl BuildingTemplate {
  pub fn new(
    name: &str,
    tiles: Vec<Vec<ObjectName>>,
    relative_door_ig: Point<InternalGrid>,
    connection_direction: Direction,
  ) -> Self {
    let height = tiles.len() as i32;
    let width = if height > 0 { tiles[0].len() as i32 } else { 0 };

    Self {
      name: name.to_string(),
      width,
      height,
      tiles,
      relative_door_ig,
      connection_direction,
    }
  }

  /// Calculates where the building's top-left corner should be placed given a connection point which is one tile away
  /// from connection point in the opposite direction.
  fn calculate_origin_ig_from_connection_point(&self, connection_point: Point<InternalGrid>) -> Point<InternalGrid> {
    let absolute_door_ig = self.calculate_absolute_door_ig(connection_point);

    Point::new_internal_grid(
      absolute_door_ig.x - self.relative_door_ig.x,
      absolute_door_ig.y - self.relative_door_ig.y,
    )
  }

  /// Calculates where the building's top-left corner should be placed given the absolute position of the door tile in
  /// internal grid coordinates.
  fn calculate_origin_ig_from_absolute_door(&self, absolute_door_ig: Point<InternalGrid>) -> Point<InternalGrid> {
    Point::new_internal_grid(
      absolute_door_ig.x - self.relative_door_ig.x,
      absolute_door_ig.y - self.relative_door_ig.y,
    )
  }

  /// Calculates the absolute position of the door tile in internal grid coordinates based on the connection point and
  /// the connection direction.
  fn calculate_absolute_door_ig(&self, connection_point: Point<InternalGrid>) -> Point<InternalGrid> {
    match self.connection_direction {
      Direction::Top => Point::new_internal_grid(connection_point.x, connection_point.y + 1),
      Direction::Bottom => Point::new_internal_grid(connection_point.x, connection_point.y - 1),
      Direction::Left => Point::new_internal_grid(connection_point.x + 1, connection_point.y),
      Direction::Right => Point::new_internal_grid(connection_point.x - 1, connection_point.y),
      _ => panic!("Invalid connection direction for building template"),
    }
  }

  fn is_placeable_at_connection(
    &self,
    connection_point: Point<InternalGrid>,
    available_space: &HashSet<Point<InternalGrid>>,
  ) -> bool {
    let building_origin_ig = self.calculate_origin_ig_from_connection_point(connection_point);

    // Don't allow buildings to be placed out of bounds
    if building_origin_ig.x < 0
      || building_origin_ig.y < 0
      || building_origin_ig.x + self.width > CHUNK_SIZE
      || building_origin_ig.y + self.height > CHUNK_SIZE
    {
      return false;
    }

    // Make sure all tiles the building will occupy are available
    for y in 0..self.height {
      for x in 0..self.width {
        let tile_ig = Point::new_internal_grid(building_origin_ig.x + x, building_origin_ig.y + y);
        if !available_space.contains(&tile_ig) {
          return false;
        }
      }
    }

    // Ensure that the door is next to the connection point and facing it
    let door_ig = Point::new_internal_grid(
      building_origin_ig.x + self.relative_door_ig.x,
      building_origin_ig.y + self.relative_door_ig.y,
    );
    let connection_point_direction: Point<InternalGrid> = self.connection_direction.to_opposite().to_point();
    let expected_door_ig = Point::new_internal_grid(
      connection_point.x + connection_point_direction.x,
      connection_point.y + connection_point_direction.y,
    );
    door_ig == expected_door_ig
  }
}

// TODO: Use metadata to cluster buildings in certain areas to create "villages"
/// The entry point for determining buildings in the object grid.
pub fn place_buildings_on_grid(object_grid: &mut ObjectGrid, settings: &Settings, metadata: &Metadata, rng: &mut StdRng) {
  let start_time = shared::get_time();
  let cg = object_grid.cg;
  if !settings.object.generate_paths || !settings.object.generate_buildings {
    debug!(
      "Skipped generating buildings for {} because it or path generation are disabled",
      cg
    );
    return;
  }
  let mut connection_points = metadata.get_connection_points_for(&cg, object_grid);
  if connection_points.len() > 1 {
    connection_points = connection_points
      .iter_mut()
      .filter(|cp| !cp.is_touching_edge())
      .map(|cp| *cp)
      .collect::<Vec<Point<InternalGrid>>>();
  }
  if connection_points.is_empty() {
    debug!(
      "No internal connection points found for {} - skipping building generation",
      cg
    );
    return;
  }

  let building_templates = get_building_templates();
  let available_grid_space = compute_available_space_map(object_grid);
  let mut occupied_grid_space = HashSet::new();
  let mut buildings_placed = 0;
  for connection_point in connection_points {
    if let Some(template) = select_fitting_building(
      &building_templates,
      connection_point,
      &available_grid_space,
      &occupied_grid_space,
      rng,
    ) {
      let absolute_door_ig = template.calculate_absolute_door_ig(connection_point);
      let building_origin_ig = template.calculate_origin_ig_from_absolute_door(absolute_door_ig);
      if place_building(&template, building_origin_ig, object_grid, &mut occupied_grid_space) {
        buildings_placed += 1;
        debug!(
          "Placed [{}] at computed origin {:?} for connection point {:?} on {}",
          template.name, building_origin_ig, connection_point, cg
        );
        update_path_in_front_of_door(&connection_point, &absolute_door_ig, object_grid);
      } else {
        warn!(
          "Failed to place [{}] at computed origin {:?} on {}",
          template.name, building_origin_ig, cg
        );
      }
    } else {
      debug!(
        "No suitable building found for connection point {:?} on {}",
        connection_point, cg
      );
    }
  }

  debug!(
    "Placed [{}] building(s) on grid for {} in {} ms on {}",
    buildings_placed,
    object_grid.cg,
    shared::get_time() - start_time,
    shared::thread_name()
  );
}

/// Returns a map of space available for placing buildings.
fn compute_available_space_map(object_grid: &mut ObjectGrid) -> HashSet<Point<InternalGrid>> {
  let mut available_space = HashSet::new();
  for y in 0..CHUNK_SIZE {
    for x in 0..CHUNK_SIZE {
      let ig = Point::new_internal_grid(x, y);
      if let Some(cell) = object_grid.get_cell_mut(&ig) {
        if !cell.is_collapsed() && cell.is_suitable_for_building_placement() {
          available_space.insert(ig);
        }
      }
    }
  }

  available_space
}

fn select_fitting_building(
  templates: &[BuildingTemplate],
  connection_point: Point<InternalGrid>,
  available_space: &HashSet<Point<InternalGrid>>,
  occupied_space: &HashSet<Point<InternalGrid>>,
  rng: &mut StdRng,
) -> Option<BuildingTemplate> {
  let mut fitting_building_templates = Vec::new();
  for template in templates {
    if template.is_placeable_at_connection(connection_point, available_space) {
      let origin_ig = template.calculate_origin_ig_from_connection_point(connection_point);
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

fn place_building(
  template: &BuildingTemplate,
  building_origin_ig: Point<InternalGrid>,
  object_grid: &mut ObjectGrid,
  occupied_space: &mut HashSet<Point<InternalGrid>>,
) -> bool {
  for y in 0..template.height {
    for x in 0..template.width {
      let ig = Point::new_internal_grid(building_origin_ig.x + x, building_origin_ig.y + y);
      let object_name = template.tiles[y as usize][x as usize];
      if let Some(cell) = object_grid.get_cell_mut(&ig) {
        cell.mark_as_collapsed(object_name);
        occupied_space.insert(ig);
      } else {
        error!(
          "Failed to get cell at {:?} for building placement on object grid for {}",
          ig, object_grid.cg
        );
        return false;
      }
    }
  }

  true
}

/// Updates the object name of the cell at the given connection point to ensure the path connects to the door
/// correctly. Without this, there would be a gap in the path leading to the door.
fn update_path_in_front_of_door(
  connection_point: &Point<InternalGrid>,
  absolute_door_ig: &Point<InternalGrid>,
  object_grid: &mut ObjectGrid,
) {
  let cg = object_grid.cg;
  if let Some(cell) = object_grid.get_cell_mut(connection_point) {
    let object_name = determine_updated_object_name(cell, connection_point, absolute_door_ig, &cg);
    cell.mark_as_collapsed(object_name);
  } else {
    error!(
      "Failed to get cell at connection point {:?} on {} to update path in front of door at {:?}",
      connection_point, cg, absolute_door_ig
    );
  }
}

/// Returns the updated object name for a path object at the given connection point based on the location of and
/// direction to the door.
fn determine_updated_object_name(
  cell: &mut Cell,
  connection_point: &Point<InternalGrid>,
  absolute_door_ig: &Point<InternalGrid>,
  cg: &Point<ChunkGrid>,
) -> ObjectName {
  let terrain_states = cell.get_possible_states();
  if !cell.is_collapsed() || terrain_states.len() != 1 {
    error!(
      "Expected collapsed path tile at connection point {:?} on {} but found: {:?} ",
      connection_point, cg, cell
    );
    return ObjectName::PathUndefined;
  }
  let terrain_state = &terrain_states[0];
  if !terrain_state.name.is_path() {
    error!(
      "Expected path tile at connection point {:?} on {} but found: {:?}",
      connection_point, cg, cell
    );
    return ObjectName::PathUndefined;
  }

  let missing_direction = Direction::from_points(connection_point, absolute_door_ig);
  let new_object_name = match (terrain_state.name, missing_direction) {
    (ObjectName::PathTop, Direction::Left) => ObjectName::PathTopLeft,
    (ObjectName::PathTop, Direction::Right) => ObjectName::PathTopRight,
    (ObjectName::PathTop, _) => ObjectName::PathVertical,
    (ObjectName::PathRight, Direction::Top) => ObjectName::PathTopRight,
    (ObjectName::PathRight, Direction::Bottom) => ObjectName::PathBottomRight,
    (ObjectName::PathRight, _) => ObjectName::PathHorizontal,
    (ObjectName::PathBottom, Direction::Left) => ObjectName::PathBottomLeft,
    (ObjectName::PathBottom, Direction::Right) => ObjectName::PathBottomRight,
    (ObjectName::PathBottom, _) => ObjectName::PathVertical,
    (ObjectName::PathLeft, Direction::Top) => ObjectName::PathTopLeft,
    (ObjectName::PathLeft, Direction::Bottom) => ObjectName::PathBottomLeft,
    (ObjectName::PathLeft, _) => ObjectName::PathHorizontal,
    (ObjectName::PathVertical, Direction::Left) => ObjectName::PathLeftVertical,
    (ObjectName::PathVertical, Direction::Right) => ObjectName::PathRightVertical,
    (ObjectName::PathHorizontal, Direction::Top) => ObjectName::PathTopHorizontal,
    (ObjectName::PathHorizontal, Direction::Bottom) => ObjectName::PathBottomHorizontal,
    (ObjectName::PathTopHorizontal, _) => ObjectName::PathCross,
    (ObjectName::PathBottomHorizontal, _) => ObjectName::PathCross,
    (ObjectName::PathLeftVertical, _) => ObjectName::PathCross,
    (ObjectName::PathRightVertical, _) => ObjectName::PathCross,
    (existing, _) => existing,
  };

  if new_object_name == terrain_state.name {
    panic!(
      "Failed to determine missing connection direction for [{:?}] tile at connection point {:?} on {} with missing direction [{:?}] and door at {} - update the match statement!",
      terrain_state.name, connection_point, cg, missing_direction, absolute_door_ig
    );
  }
  debug!(
    "Updated the object name of cell {} on {} from [{:?}] to [{:?}] because a building with a door at {} was placed",
    connection_point, cg, terrain_state.name, new_object_name, absolute_door_ig
  );

  new_object_name
}

// TODO: Move to RON file at some point
fn get_building_templates() -> Vec<BuildingTemplate> {
  vec![
    BuildingTemplate::new(
      "Small House Facing North",
      vec![
        vec![
          ObjectName::HouseSmallRoofLeft,
          ObjectName::HouseSmallRoofMiddle,
          ObjectName::HouseSmallRoofRight,
        ],
        vec![
          ObjectName::HouseSmallWallLeft,
          ObjectName::HouseSmallDoorBottom,
          ObjectName::HouseSmallWallRight,
        ],
      ],
      Point::new_internal_grid(1, 1),
      Direction::Bottom,
    ),
    BuildingTemplate::new(
      "Small House Facing West",
      vec![
        vec![
          ObjectName::HouseSmallRoofLeft,
          ObjectName::HouseSmallRoofMiddle,
          ObjectName::HouseSmallRoofRight,
        ],
        vec![
          ObjectName::HouseSmallDoorLeft,
          ObjectName::HouseSmallWallBottom,
          ObjectName::HouseSmallWallRight,
        ],
      ],
      Point::new_internal_grid(0, 1), // Reminder: First column then row
      Direction::Left,
    ),
    BuildingTemplate::new(
      "Small House Facing East",
      vec![
        vec![
          ObjectName::HouseSmallRoofLeft,
          ObjectName::HouseSmallRoofMiddle,
          ObjectName::HouseSmallRoofRight,
        ],
        vec![
          ObjectName::HouseSmallWallLeft,
          ObjectName::HouseSmallWallBottom,
          ObjectName::HouseSmallDoorRight,
        ],
      ],
      Point::new_internal_grid(2, 1), // Reminder: Column 2, row 1
      Direction::Right,
    ),
    BuildingTemplate::new(
      "Medium House Facing North",
      vec![
        vec![
          ObjectName::HouseMediumRoofLeft,
          ObjectName::HouseMediumRoofMiddle,
          ObjectName::HouseMediumRoofRight,
        ],
        vec![
          ObjectName::HouseMediumWallLeft,
          ObjectName::HouseMediumDoorBottom,
          ObjectName::HouseMediumWallRight,
        ],
      ],
      Point::new_internal_grid(1, 1),
      Direction::Bottom,
    ),
  ]
}
