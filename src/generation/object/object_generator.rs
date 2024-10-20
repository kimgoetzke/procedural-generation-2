use crate::generation::get_time;
use crate::generation::lib::{Chunk, ObjectComponent, Tile, TileData};
use crate::generation::object::lib::CollapsedCell;
use crate::generation::object::lib::ObjectGrid;
use crate::generation::object::wfc;
use crate::generation::resources::{AssetCollection, GenerationResourcesCollection, RuleSet};
use crate::resources::Settings;
use bevy::app::{App, Plugin};
use bevy::core::Name;
use bevy::hierarchy::BuildChildren;
use bevy::log::*;
use bevy::prelude::{Commands, Res, SpriteBundle, TextureAtlas, Transform};
use bevy::sprite::{Anchor, Sprite};
use rand::prelude::StdRng;
use rand::SeedableRng;

pub struct ObjectGeneratorPlugin;

impl Plugin for ObjectGeneratorPlugin {
  fn build(&self, _app: &mut App) {}
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
  spawn_data: Vec<(Chunk, Vec<TileData>)>,
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
    // TODO: Add constraints from neighbouring chunk tiles
    let mut grids = initialise_wfc_object_grids(&resources.objects.rule_sets);
    let mut collapsed_cells = vec![];

    for grid in grids.iter_mut() {
      wfc::determine_objects_in_grid(&mut rng, grid, settings);

      debug!("Completed processing of [{:?}] grid", grid.terrain);
      // TODO: Determine asset pack based on terrain or use cell's name
      collapsed_cells.extend(
        tile_data
          .iter()
          .filter_map(|tile_data| {
            grid
              .get_cell(&tile_data.flat_tile.coords.chunk_grid)
              .map(|cell_state| CollapsedCell::new(tile_data, cell_state))
          })
          .collect::<Vec<CollapsedCell>>(),
      );

      // Render tiles based on collapsed cells
      for collapsed_cell in collapsed_cells.iter() {
        let sprite_index = collapsed_cell.sprite_index;
        let tile_data = collapsed_cell.tile_data;
        let object_name = collapsed_cell.name.as_ref().expect("Failed to get object name");
        commands.entity(tile_data.entity).with_children(|parent| {
          parent.spawn(sprite(
            &tile_data.flat_tile,
            sprite_index,
            &resources.objects.path,
            Name::new(format!("{:?} Object Sprite", object_name)),
          ));
        });
      }
    }
  }

  debug!("Generated objects for chunk(s) in {} ms", get_time() - start_time);
}

fn sprite(
  tile: &Tile,
  index: i32,
  asset_collection: &AssetCollection,
  name: Name,
) -> (Name, SpriteBundle, TextureAtlas, ObjectComponent) {
  (
    name,
    SpriteBundle {
      sprite: Sprite {
        anchor: Anchor::TopLeft,
        ..Default::default()
      },
      texture: asset_collection.stat.texture.clone(),
      transform: Transform::from_xyz(
        0.,
        0.,
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
