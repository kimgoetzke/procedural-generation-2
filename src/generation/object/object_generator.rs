use crate::app_state::AppState;
use crate::constants::{CHUNK_SIZE, FOREST_OBJ_COLUMNS, SAND_OBJ_COLUMNS, TILE_SIZE};
use crate::coords::point::ChunkGrid;
use crate::coords::Point;
use crate::generation::get_time;
use crate::generation::lib::{Chunk, ObjectComponent, TerrainType, Tile, TileData, TileType};
use crate::generation::resources::{AssetCollection, GenerationResourcesCollection, Rule, RuleSet};
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

struct ObjectGrid(Vec<Vec<CellState>>);

impl ObjectGrid {
  fn new(rule_set: &RuleSet) -> Self {
    let grid = (0..CHUNK_SIZE)
      .map(|y| (0..CHUNK_SIZE).map(|x| CellState::new(x, y, rule_set)).collect())
      .collect();
    ObjectGrid(grid)
  }

  fn get_cell(&mut self, point: &Point<ChunkGrid>) -> Option<&mut CellState> {
    self.0.iter_mut().flatten().filter(|cell| cell.cg == *point).next()
  }
}

#[derive(Debug, Clone)]
struct CellState {
  cg: Point<ChunkGrid>,
  is_collapsed: bool,
  possible_states: Vec<Rule>,
  index: i32,
}

#[derive(Debug, Clone)]
struct CollapsedCell {
  tile_data: TileData,
  sprite_index: i32,
}

impl CellState {
  fn new(x: i32, y: i32, rule_set: &RuleSet) -> Self {
    CellState {
      cg: Point::new_chunk_grid(x, y),
      is_collapsed: false,
      possible_states: rule_set.rules.clone(),
      index: -1,
    }
  }

  fn set_empty(&mut self) {
    let rule = self.possible_states.get(0).unwrap().clone();
    self.index = rule.index;
    self.possible_states = vec![rule];
    self.is_collapsed = true;
  }

  fn remove_state(&mut self, index: usize) {
    self.possible_states.remove(index);
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

// TODO: Generate objects asynchronously
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
  for (_, tile_data) in spawn_data.iter_mut() {
    let mut grids = initialise_wfc_object_grids(&resources.objects.rule_sets);
    let mut collapsed_cells = vec![];

    for grid in grids.iter_mut() {
      // Collapse non-fill cells
      for tile_data in tile_data.iter_mut() {
        if let Some(cell_state) = grid.get_cell(&tile_data.flat_tile.coords.chunk_grid) {
          if tile_data.flat_tile.tile_type != TileType::Fill {
            cell_state.set_empty();
          }
        }
      }

      // Collapse random cell to start the process
      let mut rng = StdRng::seed_from_u64(settings.world.noise_seed as u64);
      let point = Point::<ChunkGrid>::new_chunk_grid(rng.gen_range(0..CHUNK_SIZE), rng.gen_range(0..CHUNK_SIZE));
      if let Some(random_cell) = grid.get_cell(&point) {
        random_cell.set_empty();
        let tile_data = tile_data
          .iter_mut()
          .find(|t| t.flat_tile.coords.chunk_grid == random_cell.cg)
          .expect(format!("Failed to find tile data for cell at {:?}", point).as_str());
        collapsed_cells.push(CollapsedCell {
          sprite_index: 0,
          tile_data: tile_data.clone(),
        });
      }

      // Sort and get the lowest entropy cell
      let mut lowest_entropy = f64::MAX;
      let mut lowest_entropy_cell = None;
      for cell in grid.0.iter_mut().flatten() {
        if !cell.is_collapsed {
          let entropy = cell.possible_states.len() as f64;
          if entropy < lowest_entropy {
            lowest_entropy = entropy;
            lowest_entropy_cell = Some(cell);
          }
        }
      }

      // Collapse cells based on neighbours
    }
  }

  for (_, tile_data) in spawn_data.iter_mut() {
    place_trees(commands, tile_data, resources, settings);
    place_stones(commands, tile_data, resources, settings);
  }
  debug!("Generated objects for chunk(s) in {} ms", get_time() - start_time);
}

// pub fn generate(
//   commands: &mut Commands,
//   spawn_data: &mut Vec<(Chunk, Vec<TileData>)>,
//   asset_collection: &Res<AssetPacksCollection>,
//   settings: &Res<Settings>,
// ) {
//   if !settings.object.generate_objects {
//     debug!("Skipped object generation because it's disabled");
//     return;
//   }
//   let start_time = get_time();
//   for (_, tile_data) in spawn_data.iter_mut() {
//     place_trees(commands, tile_data, asset_collection, settings);
//     place_stones(commands, tile_data, asset_collection, settings);
//   }
//   debug!("Generated objects for chunk(s) in {} ms", get_time() - start_time);
// }

fn place_trees(
  commands: &mut Commands,
  tile_data: &mut Vec<TileData>,
  resources: &Res<GenerationResourcesCollection>,
  settings: &Res<Settings>,
) {
  let mut rng = StdRng::seed_from_u64(settings.world.noise_seed as u64);
  generate_objects(
    commands,
    tile_data,
    &resources.objects.forest,
    TerrainType::Forest,
    settings.object.forest_obj_density,
    "Forest Object Sprite",
    FOREST_OBJ_COLUMNS as usize,
    &mut rng,
    |rng| rng.gen_range(-(TILE_SIZE as f32) / 3.0..=(TILE_SIZE as f32) / 3.0),
    |rng| rng.gen_range(-(TILE_SIZE as f32) / 3.0..=(TILE_SIZE as f32) / 3.0) + TILE_SIZE as f32,
  );
}

fn place_stones(
  commands: &mut Commands,
  tile_data: &mut Vec<TileData>,
  resources: &Res<GenerationResourcesCollection>,
  settings: &Res<Settings>,
) {
  let mut rng = StdRng::seed_from_u64(settings.world.noise_seed as u64);
  generate_objects(
    commands,
    tile_data,
    &resources.objects.sand,
    TerrainType::Sand,
    settings.object.sand_obj_density,
    "Sand Object Sprite",
    SAND_OBJ_COLUMNS as usize,
    &mut rng,
    |_| 0.,
    |_| -(TILE_SIZE as f32) / 2.,
  );
}

fn generate_objects(
  commands: &mut Commands,
  tile_data: &mut Vec<TileData>,
  asset_collection: &AssetCollection,
  terrain_type: TerrainType,
  density: f64,
  sprite_name: &str,
  columns: usize,
  rng: &mut StdRng,
  offset_x: OffsetFn,
  offset_y: OffsetFn,
) {
  let relevant_tiles: Vec<_> = tile_data
    .iter_mut()
    .filter_map(|t| {
      if t.flat_tile.terrain == terrain_type && t.flat_tile.tile_type == TileType::Fill {
        Some(t)
      } else {
        None
      }
    })
    .collect();

  for tile_data in relevant_tiles {
    if rng.gen_bool(density) {
      let offset_x = offset_x(rng);
      let offset_y = offset_y(rng);
      let index = rng.gen_range(0..columns as i32);
      trace!(
        "Placing [{}] at {:?} with offset ({}, {})",
        sprite_name,
        tile_data.flat_tile.coords.chunk_grid,
        offset_x,
        offset_y
      );
      commands.entity(tile_data.entity).with_children(|parent| {
        parent.spawn(sprite(
          &tile_data.flat_tile,
          offset_x,
          offset_y,
          index,
          asset_collection,
          Name::new(sprite_name.to_string()),
        ));
      });
    }
  }
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
