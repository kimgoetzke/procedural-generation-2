use crate::constants::*;
use crate::generation::lib::shared::CommandQueueTask;
use crate::generation::lib::{AssetCollection, Chunk, GenerationResourcesCollection, ObjectComponent, Tile, shared};
use crate::generation::object::lib::{ObjectData, ObjectGrid, ObjectName, TileData};
use crate::generation::object::wfc;
use crate::generation::object::wfc::WfcPlugin;
use crate::resources::Settings;
use bevy::app::{App, Plugin, Update};
use bevy::color::{Color, Luminance};
use bevy::ecs::world::CommandQueue;
use bevy::log::*;
use bevy::prelude::{Commands, Component, Entity, Name, Query, TextureAtlas, Transform};
use bevy::sprite::{Anchor, Sprite};
use bevy::tasks;
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on};
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};

pub struct ObjectGeneratorPlugin;

impl Plugin for ObjectGeneratorPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_plugins(WfcPlugin)
      .add_systems(Update, process_object_spawn_tasks_system);
  }
}

#[derive(Component)]
struct ObjectSpawnTask(Task<CommandQueue>);

impl CommandQueueTask for ObjectSpawnTask {
  fn poll_once(&mut self) -> Option<CommandQueue> {
    block_on(tasks::poll_once(&mut self.0))
  }
}

pub fn generate_object_grid(
  resources: &GenerationResourcesCollection,
  settings: &Settings,
  chunk: Chunk,
  chunk_entity: Entity,
) -> Option<(Chunk, Entity, ObjectGrid)> {
  let cg = chunk.coords.chunk_grid;
  if !settings.object.generate_objects {
    debug!(
      "Skipped object grid generation for {} because generating objects is disabled",
      cg
    );
    return None;
  }
  let start_time = shared::get_time();
  let grid = ObjectGrid::new_initialised(cg, &resources.objects.terrain_state_map, &chunk.layered_plane.flat);
  debug!(
    "Generated object grid for chunk {} in {} ms on {}",
    cg,
    shared::get_time() - start_time,
    shared::thread_name()
  );

  Some((chunk, chunk_entity, grid))
}

pub fn generate_object_data(
  settings: &Settings,
  object_grid: ObjectGrid,
  chunk: Chunk,
  chunk_entity: Entity,
) -> Vec<ObjectData> {
  let start_time = shared::get_time();
  let chunk_cg = chunk.coords.chunk_grid;
  let tile_data = generate_tile_data(&chunk, chunk_entity);
  let mut rng = StdRng::seed_from_u64(shared::calculate_seed(chunk_cg, settings.world.noise_seed));
  let tile_data_len = tile_data.len();
  let mut object_generation_data = (object_grid, tile_data);
  let object_data = { wfc::determine_objects_in_grid(&mut rng, &mut object_generation_data, &settings) };
  debug!(
    "Generated object data for [{}] objects, at density [{}], for chunk {} in {} ms on {}",
    object_data.len(),
    format!("{:.1}%", (object_data.len() as f32 / tile_data_len as f32) * 100.0),
    chunk_cg,
    shared::get_time() - start_time,
    shared::thread_name()
  );

  object_data
}

fn generate_tile_data(chunk: &Chunk, chunk_entity: Entity) -> Vec<TileData> {
  let mut tile_data = Vec::new();
  for t in chunk.layered_plane.flat.data.iter().flatten() {
    if let Some(tile) = t {
      tile_data.push(TileData::new(chunk_entity, tile.clone()));
    }
  }

  tile_data
}

pub fn schedule_spawning_objects(
  commands: &mut Commands,
  settings: &Settings,
  mut rng: &mut StdRng,
  object_data: Vec<ObjectData>,
) {
  let chunk_cg = object_data
    .first()
    .map_or(None, |o| Some(o.tile_data.flat_tile.coords.chunk_grid));
  let start_time = shared::get_time();
  let task_pool = AsyncComputeTaskPool::get();
  let object_data_len = object_data.len();
  for object in object_data {
    attach_object_spawn_task(commands, settings, &mut rng, task_pool, object);
  }
  if let Some(cg) = chunk_cg {
    debug!(
      "Scheduled [{}] object spawn tasks for world generation component {} in {} ms on {}",
      object_data_len,
      cg,
      shared::get_time() - start_time,
      shared::thread_name()
    );
  }
}

fn attach_object_spawn_task(
  commands: &mut Commands,
  settings: &Settings,
  mut rng: &mut StdRng,
  task_pool: &AsyncComputeTaskPool,
  object_data: ObjectData,
) {
  let sprite_index = object_data.sprite_index;
  let tile_data = object_data.tile_data.clone();
  let object_name = object_data.name.expect("Failed to get object name");
  let (offset_x, offset_y) = get_sprite_offsets(&mut rng, &object_data);
  let colour = get_randomised_colour(settings, &mut rng, &object_data);
  let task = task_pool.spawn(async move {
    let mut command_queue = CommandQueue::default();
    command_queue.push(move |world: &mut bevy::prelude::World| {
      let asset_collection = {
        let resources = shared::get_resources_from_world(world);

        resources
          .get_object_collection(
            tile_data.flat_tile.terrain,
            tile_data.flat_tile.climate,
            object_data.is_large_sprite,
            object_data.is_path_sprite,
          )
          .clone()
      };
      if let Ok(mut chunk_entity) = world.get_entity_mut(tile_data.chunk_entity) {
        chunk_entity.with_children(|parent| {
          parent.spawn(sprite(
            &tile_data.flat_tile,
            sprite_index,
            &asset_collection,
            object_name,
            offset_x,
            offset_y,
            colour,
          ));
        });
      }
    });

    command_queue
  });

  commands.spawn((Name::new("Object Spawn Task"), ObjectSpawnTask(task)));
}

// TODO: Remove or make colour randomisation look better/more visible
fn get_randomised_colour(settings: &Settings, rng: &mut StdRng, object_data: &ObjectData) -> Color {
  let base_color = Color::default();
  if object_data.is_large_sprite && settings.object.enable_colour_variations {
    let range = RGB_COLOUR_VARIATION;
    let r = (base_color.to_srgba().red + rng.random_range(-range..range)).clamp(0.0, 1.0);
    let g = (base_color.to_srgba().green + rng.random_range(-(range / 2.)..(range / 2.))).clamp(0.0, 1.0);
    let b = (base_color.to_srgba().blue + rng.random_range(-range..range)).clamp(0.0, 1.0);
    let is_darker = rng.random_bool(0.5);

    Color::srgb(r, g, b)
      .darker(if is_darker { rng.random_range(DARKNESS_RANGE) } else { 0.0 })
      .lighter(if !is_darker { rng.random_range(BRIGHTNESS_RANGE) } else { 0.0 })
  } else {
    base_color
  }
}

fn get_sprite_offsets(rng: &mut StdRng, object_data: &ObjectData) -> (f32, f32) {
  if object_data.is_large_sprite {
    (
      rng.random_range(-(TILE_SIZE as f32) / 3.0..=(TILE_SIZE as f32) / 3.0).round(),
      rng.random_range(-(TILE_SIZE as f32) / 3.0..=(TILE_SIZE as f32) / 3.0).round(),
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
  colour: Color,
) -> (Name, Sprite, Transform, ObjectComponent) {
  let base_z = (tile.coords.chunk_grid.y * CHUNK_SIZE) as f32;
  let internal_z = tile.coords.internal_grid.y as f32;
  let z = 10000. - base_z + internal_z - (offset_y / TILE_SIZE as f32);

  (
    Name::new(format!("{} {:?} Object Sprite", tile.coords.tile_grid, object_name)),
    Sprite {
      anchor: Anchor::BottomCenter,
      texture_atlas: Option::from(TextureAtlas {
        layout: asset_collection.stat.texture_atlas_layout.clone(),
        index: index as usize,
      }),
      image: asset_collection.stat.texture.clone(),
      color: colour,
      ..Default::default()
    },
    Transform::from_xyz(
      tile.coords.world.x as f32 + TILE_SIZE as f32 / 2. + offset_x,
      tile.coords.world.y as f32 + TILE_SIZE as f32 * -1. + offset_y,
      z,
    ),
    ObjectComponent {
      coords: tile.coords,
      sprite_index: index as usize,
      object_name: object_name.clone(),
      layer: z as i32,
    },
  )
}

fn process_object_spawn_tasks_system(commands: Commands, object_spawn_tasks: Query<(Entity, &mut ObjectSpawnTask)>) {
  shared::process_tasks(commands, object_spawn_tasks);
}
