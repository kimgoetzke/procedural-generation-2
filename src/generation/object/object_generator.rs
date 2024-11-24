use crate::constants::TILE_SIZE;
use crate::generation::lib::shared::CommandQueueTask;
use crate::generation::lib::{shared, Chunk, ObjectComponent, Tile, TileData};
use crate::generation::object::lib::ObjectName;
use crate::generation::object::lib::{ObjectData, ObjectGrid};
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
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};

pub struct ObjectGeneratorPlugin;

impl Plugin for ObjectGeneratorPlugin {
  fn build(&self, app: &mut App) {
    app.add_plugins(WfcPlugin).add_systems(Update, process_async_tasks_system);
  }
}

#[derive(Component)]
struct ObjectSpawnTask(Task<CommandQueue>);

impl CommandQueueTask for ObjectSpawnTask {
  fn poll_once(&mut self) -> Option<CommandQueue> {
    block_on(tasks::poll_once(&mut self.0))
  }
}

pub fn generate_object_data(
  resources: &GenerationResourcesCollection,
  settings: &Settings,
  spawn_data: (Chunk, Vec<TileData>),
) -> Vec<ObjectData> {
  if !settings.object.generate_objects {
    debug!("Skipped object generation because it's disabled");
    return vec![];
  }
  let start_time = shared::get_time();
  let chunk_cg = spawn_data.0.coords.chunk_grid;
  let grid = ObjectGrid::new_initialised(
    chunk_cg,
    &resources.objects.terrain_rules,
    &resources.objects.tile_type_rules,
    &spawn_data.1,
  );
  let mut rng = StdRng::seed_from_u64(shared::calculate_seed(chunk_cg, settings.world.noise_seed));
  let object_grid_len = grid.grid.len();
  let mut object_generation_data = (grid.clone(), spawn_data.1.clone());
  let object_data = { wfc::determine_objects_in_grid(&mut rng, &mut object_generation_data, &settings) };

  debug!(
    "Generated object data for {} objects for chunk {} in {} ms on {}",
    object_grid_len,
    chunk_cg,
    shared::get_time() - start_time,
    shared::thread_name()
  );

  object_data
}

pub fn schedule_spawning_objects(commands: &mut Commands, mut rng: &mut StdRng, object_data: Vec<ObjectData>) {
  let start_time = shared::get_time();
  let task_pool = AsyncComputeTaskPool::get();
  let object_data_len = object_data.len();
  let chunk_cg = if let Some(object_data) = object_data.first() {
    object_data.tile_data.flat_tile.coords.chunk_grid.to_string()
  } else {
    "cg(unknown)".to_string()
  };
  for object in object_data {
    attach_task_to_tile_entity(commands, &mut rng, task_pool, object);
  }
  debug!(
    "Scheduled {} object spawn tasks for chunk {} in {} ms on {}",
    object_data_len,
    chunk_cg,
    shared::get_time() - start_time,
    shared::thread_name()
  );
}

fn attach_task_to_tile_entity(
  commands: &mut Commands,
  mut rng: &mut StdRng,
  task_pool: &AsyncComputeTaskPool,
  object_data: ObjectData,
) {
  let sprite_index = object_data.sprite_index;
  let tile_data = object_data.tile_data.clone();
  let object_name = object_data.name.expect("Failed to get object name");
  let (offset_x, offset_y) = get_sprite_offsets(&mut rng, &object_data);
  let task = task_pool.spawn(async move {
    let mut command_queue = CommandQueue::default();
    command_queue.push(move |world: &mut bevy::prelude::World| {
      let asset_collection = {
        let mut system_state = SystemState::<Res<GenerationResourcesCollection>>::new(world);
        let resources = system_state.get_mut(world);
        resources
          .get_object_collection(tile_data.flat_tile.terrain, object_data.is_large_sprite)
          .clone()
      };
      if let Some(mut tile_data_entity) = world.get_entity_mut(tile_data.entity) {
        tile_data_entity.with_children(|parent| {
          parent.spawn(sprite(
            &tile_data.flat_tile,
            sprite_index,
            &asset_collection,
            object_name,
            offset_x,
            offset_y,
          ));
        });
      }
    });
    command_queue
  });

  commands.spawn((Name::new("Object Spawn Task"), ObjectSpawnTask(task)));
}

fn get_sprite_offsets(rng: &mut StdRng, object_data: &ObjectData) -> (f32, f32) {
  if object_data.is_large_sprite {
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
        200. + tile.coords.internal_grid.y as f32,
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
  shared::process_tasks(commands, object_spawn_tasks);
}
