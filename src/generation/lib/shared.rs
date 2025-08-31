use crate::coords::Point;
use crate::coords::point::ChunkGrid;
use crate::generation::lib::GenerationResourcesCollection;
use bevy::color::Color;
use bevy::ecs::component::Mutable;
use bevy::ecs::world::CommandQueue;
use bevy::prelude::{Commands, Component, Entity, Query};
use bevy_inspector_egui::egui::Color32;
use std::thread;
use std::time::SystemTime;

pub trait CommandQueueTask {
  fn poll_once(&mut self) -> Option<CommandQueue>;
}

pub fn thread_name() -> String {
  let thread = thread::current();
  let thread_name = thread.name().unwrap_or("Unnamed");
  let thread_id = thread.id();

  format!("[{} {:?}]", thread_name, thread_id)
}

pub fn process_tasks<T: CommandQueueTask + Component<Mutability = Mutable>>(
  mut commands: Commands,
  mut query: Query<(Entity, &mut T)>,
) {
  for (entity, mut task) in &mut query {
    if let Some(mut commands_queue) = task.poll_once() {
      commands.append(&mut commands_queue);
      commands.entity(entity).despawn();
    }
  }
}

pub fn get_resources_from_world(world: &mut bevy::ecs::world::World) -> GenerationResourcesCollection {
  world
    .get_resource::<GenerationResourcesCollection>()
    .expect("Failed to fetch GenerationResourcesCollection")
    .clone()
}

pub fn get_time() -> u128 {
  SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()
}

pub fn calculate_seed(cg: Point<ChunkGrid>, seed: u32) -> u64 {
  let adjusted_x = cg.x as i64 + i32::MAX as i64;
  let adjusted_y = cg.y as i64 + i32::MAX as i64;

  ((adjusted_x as u64) << 32) ^ (adjusted_y as u64) + seed as u64
}

pub fn to_colour_32(colour: Color) -> Color32 {
  let colour = colour.to_srgba();

  Color32::from_rgb((colour.red * 255.) as u8, (colour.green * 255.) as u8, (colour.blue * 255.) as u8)
}
