#![allow(dead_code)]

use crate::generation::resources::GenerationResourcesCollection;
use crate::resources::Settings;
use bevy::ecs::system::SystemState;
use bevy::ecs::world::CommandQueue;
use bevy::hierarchy::DespawnRecursiveExt;
use bevy::prelude::{Commands, Component, Entity, Query, Res};
use std::thread;

pub trait CommandQueueTask {
  fn poll_once(&mut self) -> Option<CommandQueue>;
}

pub fn get_thread_info() -> String {
  let thread = thread::current();
  let thread_name = thread.name().unwrap_or("Unnamed");
  let thread_id = thread.id();
  format!("[{} {:?}]", thread_name, thread_id)
}

pub fn process_tasks<T: CommandQueueTask + Component>(mut commands: Commands, mut query: Query<(Entity, &mut T)>) {
  for (entity, mut task) in &mut query {
    if let Some(mut commands_queue) = task.poll_once() {
      commands.append(&mut commands_queue);
      commands.entity(entity).despawn_recursive();
    }
  }
}

pub fn get_resources_and_settings(world: &mut bevy::ecs::world::World) -> (GenerationResourcesCollection, Settings) {
  let (resources, settings) = {
    let mut system_state = SystemState::<(Res<GenerationResourcesCollection>, Res<Settings>)>::new(world);
    let (resources, settings) = system_state.get_mut(world);
    (resources.clone(), settings.clone())
  };
  (resources, settings)
}
