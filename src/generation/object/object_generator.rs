use crate::constants::TILE_SIZE;
use crate::generation::get_time;
use crate::generation::lib::{Chunk, ObjectComponent, Tile, TileData};
use crate::generation::object::components::{ObjectGenerationDataComponent, ObjectGenerationStatus};
use crate::generation::object::lib::ObjectGrid;
use crate::generation::object::lib::{CollapsedCell, ObjectName};
use crate::generation::object::wfc;
use crate::generation::object::wfc::WfcPlugin;
use crate::generation::resources::{AssetCollection, GenerationResourcesCollection};
use crate::resources::Settings;
use bevy::app::{App, Plugin, Update};
use bevy::core::Name;
use bevy::hierarchy::BuildChildren;
use bevy::log::*;
use bevy::prelude::{Commands, Entity, Query, Res, SpriteBundle, TextureAtlas, Transform};
use bevy::sprite::{Anchor, Sprite};
use bevy::tasks::AsyncComputeTaskPool;
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};

pub struct ObjectGeneratorPlugin;

impl Plugin for ObjectGeneratorPlugin {
  fn build(&self, app: &mut App) {
    app.add_plugins(WfcPlugin).add_systems(Update, generate_objects_system);
  }
}

pub fn generate(
  spawn_data: Vec<(Chunk, Vec<TileData>)>,
  resources: &Res<GenerationResourcesCollection>,
  settings: &Res<Settings>,
  commands: &mut Commands,
) {
  if !settings.object.generate_objects {
    debug!("Skipped object generation because it's disabled");
    return;
  }
  let start_time = get_time();
  for (chunk, tile_data) in spawn_data.iter() {
    let grid = ObjectGrid::new_initialised(
      &resources.objects.terrain_rules,
      &resources.objects.tile_type_rules,
      tile_data,
    );
    commands.spawn((
      Name::new(format!(
        "Object Generation Data for Chunk w{} wg{}",
        chunk.coords.world, chunk.coords.world_grid
      )),
      ObjectGenerationDataComponent::new(grid.clone(), tile_data.clone()),
    ));
  }
  debug!(
    "Generated object generation data for chunk(s) in {} ms",
    get_time() - start_time
  );
}

// TODO: Determine objects and spawn sprites asynchronously
//   - Pick a chunk to generate objects for
//   - Run system that runs x iterations of the WFC algorithm per frame (allow debugging if possible)
//   - When done for a chunk, schedule spawning all objects for that chunk
// TODO: Remove paths and create new algorithm for them
fn generate_objects_system(
  mut commands: Commands,
  mut query: Query<(Entity, &mut ObjectGenerationDataComponent)>,
  resources: Res<GenerationResourcesCollection>,
  settings: Res<Settings>,
) {
  for (entity, mut component) in query.iter_mut() {
    if component.status == ObjectGenerationStatus::Done || component.status == ObjectGenerationStatus::Failure {
      commands.entity(entity).despawn();
      continue;
    }

    let start_time = get_time();

    if component.status == ObjectGenerationStatus::Calculating {
      let task_pool = AsyncComputeTaskPool::get();
      let task = task_pool.spawn(async move {
        let mut rng = StdRng::seed_from_u64(settings.world.noise_seed as u64);
        let collapsed_cells = wfc::determine_objects_in_grid(&mut rng, &mut component, &settings);
        component.collapsed_cells = Some(collapsed_cells);
      });
    }

    if component.status == ObjectGenerationStatus::Calculating {
      // Render tiles based on collapsed cells
      for collapsed_cell in component.collapsed_cells.iter() {
        let mut rng = StdRng::seed_from_u64(settings.world.noise_seed as u64);
        let sprite_index = collapsed_cell.sprite_index;
        let tile_data = collapsed_cell.tile_data;
        let object_name = collapsed_cell.name.expect("Failed to get object name");
        let asset_collection = resources.get_object_collection(tile_data.flat_tile.terrain, collapsed_cell.is_large_sprite);
        let (offset_x, offset_y) = get_sprite_offsets(&mut rng, collapsed_cell);
        commands.entity(tile_data.entity).with_children(|parent| {
          parent.spawn(sprite(
            &tile_data.flat_tile,
            sprite_index,
            asset_collection,
            object_name,
            offset_x,
            offset_y,
          ));
        });
      }

      debug!(
        "Generated {} objects for entity #{} in {} ms",
        component.collapsed_cells.len(),
        entity,
        get_time() - start_time
      );
    }
  }
}

fn get_sprite_offsets(rng: &mut StdRng, collapsed_cell: &CollapsedCell) -> (f32, f32) {
  if collapsed_cell.is_large_sprite {
    (
      rng.gen_range(-(TILE_SIZE as f32) / 3.0..=(TILE_SIZE as f32) / 3.0),
      rng.gen_range(-(TILE_SIZE as f32) / 3.0..=(TILE_SIZE as f32) / 3.0),
    )
  } else {
    (0., 0.)
  }
}

fn sprite(
  tile: &Tile,
  index: i32,
  asset_collection: &AssetCollection,
  object_name: ObjectName,
  offset_x: f32,
  offset_y: f32,
) -> (Name, SpriteBundle, TextureAtlas, ObjectComponent) {
  (
    Name::new(format!("{:?} Object Sprite", object_name)),
    SpriteBundle {
      sprite: Sprite {
        anchor: Anchor::BottomCenter,
        ..Default::default()
      },
      texture: asset_collection.stat.texture.clone(),
      transform: Transform::from_xyz(
        TILE_SIZE as f32 / 2. + offset_x,
        TILE_SIZE as f32 * -1. + offset_y,
        // TODO: Incorporate the chunk itself in the z-axis as it any chunk will render on top of the chunk below it
        200. + tile.coords.chunk_grid.y as f32,
      ),
      ..Default::default()
    },
    TextureAtlas {
      layout: asset_collection.stat.texture_atlas_layout.clone(),
      index: index as usize,
    },
    ObjectComponent {
      coords: tile.coords,
      sprite_index: index as usize,
      object_name: object_name.clone(),
    },
  )
}
