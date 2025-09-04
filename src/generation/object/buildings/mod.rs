use crate::constants::CHUNK_SIZE;
use crate::coords::Point;
use crate::coords::point::InternalGrid;
use crate::generation::lib::{Direction, shared};
use crate::generation::object::lib::{ObjectGrid, ObjectName};
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
  relative_door_ig: Point<InternalGrid>,
  connection_direction: Direction,
}

impl BuildingTemplate {
  pub fn new(name: &str, tiles: Vec<Vec<ObjectName>>, door_x: i32, door_y: i32, connection_direction: Direction) -> Self {
    let height = tiles.len() as i32;
    let width = if height > 0 { tiles[0].len() as i32 } else { 0 };

    Self {
      name: name.to_string(),
      width,
      height,
      tiles,
      relative_door_ig: Point::new_internal_grid(door_x, door_y),
      connection_direction,
    }
  }

  /// Calculates where the building's top-left corner should be placed given a connection point.
  fn calculate_building_top_left_ig(&self, connection_point: Point<InternalGrid>) -> Point<InternalGrid> {
    // Calculate where the door should be (one tile away from connection point in the opposite direction)
    let door_ig = match self.connection_direction {
      Direction::Top => Point::new_internal_grid(connection_point.x, connection_point.y + 1),
      Direction::Bottom => Point::new_internal_grid(connection_point.x, connection_point.y - 1),
      Direction::Left => Point::new_internal_grid(connection_point.x + 1, connection_point.y),
      Direction::Right => Point::new_internal_grid(connection_point.x - 1, connection_point.y),
      _ => panic!("Invalid connection direction for building template"),
    };

    Point::new_internal_grid(door_ig.x - self.relative_door_ig.x, door_ig.y - self.relative_door_ig.y)
  }

  fn can_place_at_connection(
    &self,
    connection_point: Point<InternalGrid>,
    available_space: &HashSet<Point<InternalGrid>>,
  ) -> bool {
    let building_origin = self.calculate_building_top_left_ig(connection_point);

    // Check chunk boundaries
    if building_origin.x < 0
      || building_origin.y < 0
      || building_origin.x + self.width > CHUNK_SIZE
      || building_origin.y + self.height > CHUNK_SIZE
    {
      return false;
    }

    // Check if all building tiles are available
    for y in 0..self.height {
      for x in 0..self.width {
        let tile_pos = Point::new_internal_grid(building_origin.x + x, building_origin.y + y);
        if !available_space.contains(&tile_pos) {
          return false;
        }
      }
    }

    // Verify the connection point is adjacent to where the door will be
    let door_world_position = Point::new_internal_grid(
      building_origin.x + self.relative_door_ig.x,
      building_origin.y + self.relative_door_ig.y,
    );

    self.is_adjacent(door_world_position, connection_point)
  }

  fn is_adjacent(&self, reference_ig: Point<InternalGrid>, other_ig: Point<InternalGrid>) -> bool {
    let dx = (reference_ig.x - other_ig.x).abs();
    let dy = (reference_ig.y - other_ig.y).abs();

    (dx == 1 && dy == 0) || (dx == 0 && dy == 1)
  }
}

// TODO: Override path tiles in front of doors to ensure seamless connectivity
// TODO: Make sure buildings face the correct direction based on path direction
// TODO: Make it possible for buildings to spawn at single edge connection points with no connections
/// The entry point for determining buildings in the object grid.
pub fn place_buildings_on_grid(object_grid: &mut ObjectGrid, settings: &Settings, metadata: &Metadata, rng: &mut StdRng) {
  let start_time = shared::get_time();
  let cg = object_grid.cg;
  if !settings.object.generate_paths {
    debug!("Skipped generating buildings for {} because path generation is disabled", cg);
    return;
  }
  let connection_points = metadata
    .get_connection_points_for(&cg, object_grid)
    .iter_mut()
    .filter(|cp| !cp.is_touching_edge())
    .map(|cp| *cp)
    .collect::<Vec<Point<InternalGrid>>>();
  if connection_points.is_empty() {
    debug!(
      "No internal connection points found for {} - skipping building generation",
      cg
    );
    return;
  }

  let available_space = compute_available_space_map(object_grid);
  let building_templates = get_building_templates();
  let mut buildings_placed = 0;
  let mut occupied_positions = HashSet::new();
  for connection_point in connection_points {
    if let Some(template) = select_fitting_building(
      &building_templates,
      connection_point,
      &available_space,
      &occupied_positions,
      rng,
    ) {
      let building_origin = template.calculate_building_top_left_ig(connection_point);
      if place_building(object_grid, &template, building_origin, &mut occupied_positions) {
        buildings_placed += 1;
        debug!(
          "Placed [{}] at computed origin {:?} for connection point {:?} on {}",
          template.name, building_origin, connection_point, cg
        );
      } else {
        warn!(
          "Failed to place [{}] at computed origin {:?} on {}",
          template.name, building_origin, cg
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
fn compute_available_space_map(object_grid: &ObjectGrid) -> HashSet<Point<InternalGrid>> {
  let mut available_space = HashSet::new();
  for y in 0..CHUNK_SIZE {
    for x in 0..CHUNK_SIZE {
      let ig = Point::new_internal_grid(x, y);
      if let Some(cell) = object_grid.get_cell(&ig) {
        if cell.is_walkable() && !cell.is_collapsed() {
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
  let mut fitting_templates = Vec::new();
  for template in templates {
    if template.can_place_at_connection(connection_point, available_space) {
      let building_origin = template.calculate_building_top_left_ig(connection_point);
      let mut is_overlapping = false;
      for y in 0..template.height {
        for x in 0..template.width {
          let ig = Point::new_internal_grid(building_origin.x + x, building_origin.y + y);
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
        fitting_templates.push(template.clone());
      }
    }
  }

  if fitting_templates.is_empty() {
    return None;
  }

  let selected_index = rng.random_range(0..fitting_templates.len());

  Some(fitting_templates[selected_index].clone())
}

fn place_building(
  object_grid: &mut ObjectGrid,
  template: &BuildingTemplate,
  building_origin: Point<InternalGrid>,
  occupied_positions: &mut HashSet<Point<InternalGrid>>,
) -> bool {
  for y in 0..template.height {
    for x in 0..template.width {
      let ig = Point::new_internal_grid(building_origin.x + x, building_origin.y + y);
      let object_name = template.tiles[y as usize][x as usize];
      if let Some(cell) = object_grid.get_cell_mut(&ig) {
        cell.mark_as_collapsed(object_name);
        occupied_positions.insert(ig);
      } else {
        error!("Failed to get cell at {:?} for building placement", ig);
        return false;
      }
    }
  }

  true
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
      1,
      1,
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
      1,
      0,
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
      1,
      2,
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
      1,
      1,
      Direction::Bottom,
    ),
  ]
}
