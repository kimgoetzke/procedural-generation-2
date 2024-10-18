use crate::constants::TILE_SIZE;
use crate::generation::get_time;
use crate::generation::lib::{Chunk, ObjectComponent, Tile, TileData};
use crate::generation::object::lib::Cell;
use crate::generation::object::lib::ObjectGrid;
use crate::generation::resources::{AssetCollection, GenerationResourcesCollection, RuleSet};
use crate::resources::Settings;
use bevy::app::{App, Plugin};
use bevy::core::Name;
use bevy::hierarchy::BuildChildren;
use bevy::log::*;
use bevy::prelude::{Commands, Res, SpriteBundle, TextureAtlas, Transform};
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};

pub struct ObjectGeneratorPlugin;

impl Plugin for ObjectGeneratorPlugin {
  fn build(&self, _app: &mut App) {}
}

type OffsetFn = fn(&mut StdRng) -> f32;

#[derive(Debug, Clone)]
struct CollapsedCell<'a> {
  tile_data: &'a TileData,
  sprite_index: i32,
}

impl<'a> CollapsedCell<'a> {
  fn new(tile_data: &'a TileData, cell_state: &Cell) -> Self {
    let sprite_index = cell_state.index;
    if sprite_index == -1 {
      error!(
        "Creating collapsed cell from non-collapsed cell state at cg{:?}",
        cell_state.cg
      );
    }
    CollapsedCell { tile_data, sprite_index }
  }
}

fn initialise_wfc_object_grids(rule_sets: &Vec<RuleSet>) -> Vec<ObjectGrid> {
  let mut grids = vec![];
  for rule_set in rule_sets.iter() {
    let grid = ObjectGrid::new(rule_set);
    grids.push(grid);
  }

  grids
}

// TODO: Generate objects asynchronously later
pub fn generate(
  commands: &mut Commands,
  mut spawn_data: Vec<(Chunk, Vec<TileData>)>,
  resources: &Res<GenerationResourcesCollection>,
  settings: &Res<Settings>,
) {
  if !settings.object.generate_objects {
    debug!("Skipped object generation because it's disabled");
    return;
  }
  let start_time = get_time();
  let mut rng = StdRng::seed_from_u64(settings.world.noise_seed as u64);

  for (_, tile_data) in spawn_data.iter() {
    let mut grids = initialise_wfc_object_grids(&resources.objects.rule_sets);
    let mut collapsed_cells = vec![];

    for grid in grids.iter_mut() {
      // // Collapse non-fill cells
      // for tile_data in tile_data.iter() {
      //   if let Some(cell_state) = grid.get_cell_mut(&tile_data.flat_tile.coords.chunk_grid) {
      //     if tile_data.flat_tile.tile_type != TileType::Fill {
      //       collapsed_cells.push(CollapsedCell::new(tile_data, cell_state.collapse_to_empty()));
      //     }
      //   }
      // }

      let mut wave_count = 0;
      while !process_wave(&mut rng, grid) {
        wave_count += 1;
        debug!("Processed [{:?}] grid wave {}", grid.terrain, wave_count);
        continue;
      }
      collapsed_cells.extend(
        tile_data
          .iter()
          .filter_map(|tile_data| {
            grid
              .get_cell_mut(&tile_data.flat_tile.coords.chunk_grid)
              .map(|cell_state| CollapsedCell::new(tile_data, cell_state))
          })
          .collect::<Vec<CollapsedCell>>(),
      );

      // Render tiles based on collapsed cells
      for collapsed_cell in collapsed_cells.iter() {
        let sprite_index = collapsed_cell.sprite_index;
        let tile_data = &collapsed_cell.tile_data;
        commands.entity(tile_data.entity).with_children(|parent| {
          parent.spawn(sprite(
            &tile_data.flat_tile,
            0.,
            0.,
            sprite_index,
            &resources.objects.path,
            Name::new("WFC Sprite".to_string()),
          ));
        });
      }
    }
  }

  // for (_, tile_data) in spawn_data.iter_mut() {
  //   place_trees(commands, tile_data, resources, settings);
  //   place_stones(commands, tile_data, resources, settings);
  // }
  debug!("Generated objects for chunk(s) in {} ms", get_time() - start_time);
}

fn process_wave(mut rng: &mut StdRng, grid: &mut ObjectGrid) -> bool {
  // Sort and get the lowest entropy cell
  let lowest_entropy_cells = grid.get_cells_with_lowest_entropy();
  if lowest_entropy_cells.is_empty() {
    info!("No more cells to collapse in this [{:?}] grid", grid.terrain);
    return true;
  }

  // Collapse random cell from the cells with the lowest entropy
  let index = rng.gen_range(0..lowest_entropy_cells.len());
  let random_cell: &Cell = lowest_entropy_cells.get(index).expect("Failed to get random cell");
  let mut random_cell_clone = random_cell.clone();
  random_cell_clone.collapse(&mut rng);

  // Update all neighbours
  let mut stack: Vec<Cell> = vec![random_cell_clone];
  while let Some(cell) = stack.pop() {
    grid.set_cell(cell.clone());
    let mut neighbours = grid.get_neighbours(&cell.cg);
    for (connection, neighbour) in neighbours.iter_mut() {
      if !neighbour.is_collapsed {
        let (has_changed, neighbour_cell_state) = neighbour.clone_and_update(&cell, &connection);
        if has_changed {
          trace!(
            "Updated cell at cg{:?} in with {:?} new state(s)",
            neighbour.cg,
            neighbour_cell_state.possible_states.len()
          );
          stack.push(neighbour_cell_state);
        }
      }
    }
  }

  false
}

fn sprite(
  tile: &Tile,
  offset_x: f32,
  offset_y: f32,
  index: i32,
  asset_collection: &AssetCollection,
  name: Name,
) -> (Name, SpriteBundle, TextureAtlas, ObjectComponent) {
  (
    name,
    SpriteBundle {
      texture: asset_collection.stat.texture.clone(),
      transform: Transform::from_xyz(
        offset_x + TILE_SIZE as f32 / 2.0,
        offset_y,
        // TODO: Incorporate the chunk itself in the z-axis as it any chunk will render on top of the chunk below it
        200. + tile.coords.chunk_grid.y as f32,
      ),
      ..Default::default()
    },
    TextureAtlas {
      layout: asset_collection.stat.texture_atlas_layout.clone(),
      index: index as usize,
    },
    ObjectComponent {},
  )
}
