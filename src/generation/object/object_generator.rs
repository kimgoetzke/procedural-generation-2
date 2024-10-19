use crate::constants::TILE_SIZE;
use crate::generation::get_time;
use crate::generation::lib::{Chunk, ObjectComponent, Tile, TileData};
use crate::generation::object::lib::ObjectGrid;
use crate::generation::object::lib::{Cell, NoPossibleStatesFailure};
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
    // TODO: Consider creating single grid only and adding rule sets based on terrain
    let mut grids = initialise_wfc_object_grids(&resources.objects.rule_sets);
    let mut collapsed_cells = vec![];

    for mut grid in grids.iter_mut() {
      let mut wave_count = 0;
      let mut has_entropy = true;
      let mut snapshots = vec![];
      let mut error_count = 0;

      while has_entropy {
        match process_wave(&mut rng, grid) {
          Ok(is_done) => {
            let grid_clone = grid.clone();
            snapshots.push(grid_clone);
            wave_count += 1;
            debug!("Processed [{:?}] grid wave {}", grid.terrain, wave_count);
            has_entropy = !is_done;
          }
          Err(_) => {
            error_count += 1;
            let snapshot_index = snapshots.len() - error_count;
            let error_message = format!("Failed to get snapshot {}", snapshot_index.to_string());
            grid.restore_from_snapshot(snapshots.get(snapshot_index).expect(error_message.as_str()));
            warn!(
              "Failed to reduce entropy in wave {} in [{:?}] grid - restored snapshot {}/{}",
              wave_count + 1,
              grid.terrain,
              snapshot_index,
              snapshots.len()
            );
          }
        }
      }

      // TODO: Determine asset pack based on terrain or use cell's name
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
            Name::new("WFC Sprite".to_string()), // TODO: Add name to sprite
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

// TODO: Add snapshots at each wave so progress can be reset if an impossible state is reached
fn process_wave(mut rng: &mut StdRng, grid: &mut ObjectGrid) -> Result<bool, NoPossibleStatesFailure> {
  // Sort and get the lowest entropy cell
  let lowest_entropy_cells = grid.get_cells_with_lowest_entropy();
  if lowest_entropy_cells.is_empty() {
    info!("No more cells to collapse in this [{:?}] grid", grid.terrain);
    return Ok(true);
  }

  // Collapse random cell from the cells with the lowest entropy
  let index = rng.gen_range(0..lowest_entropy_cells.len());
  let random_cell: &Cell = lowest_entropy_cells.get(index).expect("Failed to get random cell");
  let mut random_cell_clone = random_cell.clone();
  random_cell_clone.collapse(&mut rng);

  // Update every neighbours' states
  let mut stack: Vec<Cell> = vec![random_cell_clone];
  while let Some(cell) = stack.pop() {
    grid.set_cell(cell.clone());
    let mut neighbours = grid.get_neighbours(&cell.cg);
    for (connection, neighbour) in neighbours.iter_mut() {
      if !neighbour.is_collapsed {
        if let Ok((has_changed, neighbour_cell)) = neighbour.clone_and_reduce(&cell, &connection) {
          if has_changed {
            stack.push(neighbour_cell);
          }
        } else {
          return Err(NoPossibleStatesFailure {});
        }
      }
    }
  }

  Ok(false)
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
