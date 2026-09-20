use crate::coordinates::Point;
use crate::coordinates::point::{ChunkGrid, InternalGrid};
use crate::generation::object::lib::{BuildingTemplate, ObjectGrid};
use crate::generation::object::settlements::settlement_generation::{
  select_fitting_building, update_path_in_front_of_entrance,
};
use bevy::log::*;
use bevy::platform::collections::HashSet;
use rand::prelude::StdRng;

pub(super) fn place_buildings(
  object_grid: &mut ObjectGrid,
  path_points: &[Point<InternalGrid>],
  available_grid_space: &HashSet<Point<InternalGrid>>,
  building_templates: &[BuildingTemplate],
  occupied_grid_space: &mut HashSet<Point<InternalGrid>>,
  rng: &mut StdRng,
  cg: Point<ChunkGrid>,
) -> i8 {
  let mut buildings_placed = 0;
  for &path_ig in path_points {
    if let Some(building_template) =
      select_fitting_building(building_templates, path_ig, available_grid_space, occupied_grid_space, rng)
    {
      let absolute_door_ig = building_template.calculate_absolute_door_ig(path_ig);
      let building_origin_ig = building_template.calculate_origin_ig_from_absolute_door(absolute_door_ig);
      if place_building(&building_template, rng, building_origin_ig, object_grid, occupied_grid_space) {
        buildings_placed += 1;
        trace!(
          "Placed [{}] with origin {:?} for path point {:?} on {}",
          building_template.id, building_origin_ig, path_ig, cg
        );
        update_path_in_front_of_entrance(&path_ig, &absolute_door_ig, object_grid);
      } else {
        warn!(
          "Failed to place [{}] with origin {:?} on {}",
          building_template.id, building_origin_ig, cg
        );
      }
    } else {
      trace!("No suitable building found for path point {:?} on {}", path_ig, cg);
    }
  }

  buildings_placed
}

fn place_building(
  building_template: &BuildingTemplate,
  rng: &mut StdRng,
  building_origin_ig: Point<InternalGrid>,
  object_grid: &mut ObjectGrid,
  occupied_space: &mut HashSet<Point<InternalGrid>>,
) -> bool {
  let tiles = building_template.generate_tiles(rng);
  for y in 0..building_template.height {
    for x in 0..building_template.width {
      let ig = Point::new_internal_grid(building_origin_ig.x + x, building_origin_ig.y + y);
      let object_name = tiles[y as usize][x as usize];
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
