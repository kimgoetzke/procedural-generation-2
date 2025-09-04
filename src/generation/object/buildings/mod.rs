use crate::constants::CHUNK_SIZE;
use crate::coords::Point;
use crate::coords::point::InternalGrid;
use crate::generation::lib::shared;
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
pub struct BuildingTemplate {
  pub name: String,
  pub width: i32,
  pub height: i32,
  pub tiles: Vec<Vec<ObjectName>>,
  pub priority: u8,
  pub min_distance_from_connection: i32,
}

impl BuildingTemplate {
  pub fn new(name: &str, tiles: Vec<Vec<ObjectName>>, priority: u8, min_distance: i32) -> Self {
    let height = tiles.len() as i32;
    let width = if height > 0 { tiles[0].len() as i32 } else { 0 };

    Self {
      name: name.to_string(),
      width,
      height,
      tiles,
      priority,
      min_distance_from_connection: min_distance,
    }
  }
}

/// The entry point for determining buildings in the object grid.
pub fn determine_buildings(settings: &Settings, metadata: &Metadata, rng: &mut StdRng, object_grid: &mut ObjectGrid) {
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

  let building_templates = get_building_templates();
  let mut sorted_building_templates = building_templates;
  sorted_building_templates.sort_by(|a, b| b.priority.cmp(&a.priority));

  let mut placed_buildings = HashSet::new();
  let mut attempts = 1;
  let max_attempts = 10;
  for template in &sorted_building_templates {
    if attempts >= max_attempts {
      break;
    }

    // Try to place this building type near connection points
    let placement_candidates = find_placement_candidates(object_grid, &connection_points, template, &placed_buildings);

    if !placement_candidates.is_empty() {
      // Randomly select from valid candidates
      let selected_position = placement_candidates[rng.random_range(0..placement_candidates.len())];

      if place_building(object_grid, template, selected_position) {
        // Mark area as occupied
        for y in 0..template.height {
          for x in 0..template.width {
            placed_buildings.insert(Point::new_internal_grid(selected_position.x + x, selected_position.y + y));
          }
        }
        debug!("Placed [{}] at {:?} on {}", template.name, selected_position, cg);
      }
    }

    attempts += 1;
  }

  debug!(
    "Completed generating buildings for {} in {} ms on {}",
    object_grid.cg,
    shared::get_time() - start_time,
    shared::thread_name()
  );
}

fn find_placement_candidates(
  object_grid: &ObjectGrid,
  connection_points: &[Point<InternalGrid>],
  template: &BuildingTemplate,
  occupied_positions: &HashSet<Point<InternalGrid>>,
) -> Vec<Point<InternalGrid>> {
  let mut candidates = Vec::new();

  for &cp in connection_points {
    // Search in expanding rings around each connection point
    for radius in template.min_distance_from_connection..=6 {
      let ring_positions = get_positions_at_distance(cp, radius);
      for pos in ring_positions {
        if can_place_building(object_grid, template, pos, occupied_positions) {
          candidates.push(pos);
        }
      }
      if !candidates.is_empty() {
        break;
      }
    }
  }

  candidates
}

fn get_positions_at_distance(center: Point<InternalGrid>, distance: i32) -> Vec<Point<InternalGrid>> {
  let mut positions = Vec::new();

  // Generate positions in a square ring at the given distance
  for dx in -distance..=distance {
    for dy in -distance..=distance {
      // Only include positions at exactly the target distance (Manhattan or Chebyshev)
      if dx.abs().max(dy.abs()) == distance {
        let pos = Point::new_internal_grid(center.x + dx, center.y + dy);
        if is_valid_position(pos) {
          positions.push(pos);
        }
      }
    }
  }

  positions
}

fn is_valid_position(pos: Point<InternalGrid>) -> bool {
  pos.x >= 0 && pos.y >= 0 && pos.x < CHUNK_SIZE && pos.y < CHUNK_SIZE
}

fn can_place_building(
  object_grid: &ObjectGrid,
  template: &BuildingTemplate,
  top_left: Point<InternalGrid>,
  occupied_positions: &HashSet<Point<InternalGrid>>,
) -> bool {
  // Check if building fits within chunk boundaries
  if top_left.x + template.width > CHUNK_SIZE || top_left.y + template.height > CHUNK_SIZE {
    return false;
  }

  // Check if all required cells are available
  for y in 0..template.height {
    for x in 0..template.width {
      let pos = Point::new_internal_grid(top_left.x + x, top_left.y + y);

      // Check if position is already occupied by another building
      if occupied_positions.contains(&pos) {
        return false;
      }

      // Check if the cell exists and is suitable for building
      if let Some(cell) = object_grid.get_cell(&pos) {
        // Cell must be walkable and not already collapsed
        if !cell.is_walkable() || cell.is_collapsed() {
          return false;
        }
      } else {
        return false;
      }
    }
  }

  true
}

fn place_building(object_grid: &mut ObjectGrid, template: &BuildingTemplate, top_left: Point<InternalGrid>) -> bool {
  // Verify placement is still valid
  if !can_place_building(object_grid, template, top_left, &HashSet::new()) {
    return false;
  }

  // Place all building tiles
  for y in 0..template.height {
    for x in 0..template.width {
      let pos = Point::new_internal_grid(top_left.x + x, top_left.y + y);
      let object_name = template.tiles[y as usize][x as usize];

      if let Some(cell) = object_grid.get_cell_mut(&pos) {
        cell.set_collapsed(object_name);
      } else {
        error!("Failed to get cell at {:?} for building placement", pos);
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
      5,
      1,
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
      5,
      1,
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
      5,
      1,
    ),
    BuildingTemplate::new(
      "Medium House",
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
      10,
      1,
    ),
  ]
}
