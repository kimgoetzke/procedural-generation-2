use crate::constants::TILE_SIZE;
use crate::generation;
use crate::generation::async_utils::AsyncTask;
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
use bevy::ecs::system::SystemState;
use bevy::ecs::world::CommandQueue;
use bevy::hierarchy::BuildWorldChildren;
use bevy::log::*;
use bevy::prelude::{Commands, Component, Entity, Query, Res, SpriteBundle, TextureAtlas, Transform};
use bevy::sprite::{Anchor, Sprite};
use bevy::tasks;
use bevy::tasks::{block_on, AsyncComputeTaskPool, Task};
use generation::async_utils;
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};

pub struct ObjectGeneratorPlugin;

impl Plugin for ObjectGeneratorPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_plugins(WfcPlugin)
      .add_systems(Update, (generate_objects_system, process_async_tasks_system));
  }
}

#[derive(Component)]
struct ObjectSpawnTask(Task<CommandQueue>);

impl AsyncTask for ObjectSpawnTask {
  fn poll_once(&mut self) -> Option<CommandQueue> {
    block_on(tasks::poll_once(&mut self.0))
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
    "Generated object generation data for chunk(s) in {} ms on {}",
    get_time() - start_time,
    async_utils::get_thread_info()
  );
}

// TODO: Remove paths and create new algorithm for them
fn generate_objects_system(
  mut commands: Commands,
  mut query: Query<(Entity, &mut ObjectGenerationDataComponent)>,
  settings: Res<Settings>,
) {
  for (entity, mut component) in query.iter_mut() {
    if component.status == ObjectGenerationStatus::Done {
      commands.entity(entity).despawn();
      continue;
    }

    if component.status == ObjectGenerationStatus::Pending {
      let start_time = get_time();
      let mut rng = StdRng::seed_from_u64(settings.world.noise_seed as u64);
      let task_pool = AsyncComputeTaskPool::get();
      let object_grid_len = component.object_grid.grid.len();
      let entity_id = entity;
      let collapsed_cells = { wfc::determine_objects_in_grid(&mut rng, &mut *component, &settings) };
      for collapsed_cell in collapsed_cells {
        attach_task_to_tile_entity(&mut commands, &mut rng, task_pool, collapsed_cell);
      }
      debug!(
        "Determined objects and scheduled {} objects spawn tasks for entity #{} in {} ms on {}",
        object_grid_len,
        entity_id,
        get_time() - start_time,
        async_utils::get_thread_info()
      );
    }
  }
}

fn attach_task_to_tile_entity(
  commands: &mut Commands,
  mut rng: &mut StdRng,
  task_pool: &AsyncComputeTaskPool,
  collapsed_cell: CollapsedCell,
) {
  let sprite_index = collapsed_cell.sprite_index;
  let tile_data = collapsed_cell.tile_data.clone();
  let object_name = collapsed_cell.name.expect("Failed to get object name");
  let (offset_x, offset_y) = get_sprite_offsets(&mut rng, &collapsed_cell);
  let task = task_pool.spawn(async move {
    let mut command_queue = CommandQueue::default();
    command_queue.push(move |world: &mut bevy::prelude::World| {
      let asset_collection = {
        let mut system_state = SystemState::<Res<GenerationResourcesCollection>>::new(world);
        let resources = system_state.get_mut(world);
        resources
          .get_object_collection(tile_data.flat_tile.terrain, collapsed_cell.is_large_sprite)
          .clone()
      };
      world.entity_mut(tile_data.entity).with_children(|parent| {
        parent.spawn(sprite(
          &tile_data.flat_tile,
          sprite_index,
          &asset_collection,
          object_name,
          offset_x,
          offset_y,
        ));
      });
    });
    command_queue
  });

  commands.spawn((Name::new("Object Spawn Task"), ObjectSpawnTask(task)));
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

fn process_async_tasks_system(commands: Commands, object_spawn_tasks: Query<(Entity, &mut ObjectSpawnTask)>) {
  async_utils::process_tasks(commands, object_spawn_tasks);
}
