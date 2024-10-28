#![allow(dead_code)]

use bevy::ecs::world::CommandQueue;
use bevy::hierarchy::DespawnRecursiveExt;
use bevy::prelude::{Commands, Component, Entity, Query};
use std::thread;

pub trait AsyncTask {
  fn poll_once(&mut self) -> Option<CommandQueue>;
}

pub fn get_thread_info() -> String {
  let thread = thread::current();
  let thread_name = thread.name().unwrap_or("Unnamed");
  let thread_id = thread.id();
  format!("[{} {:?}]", thread_name, thread_id)
}

pub fn process_tasks<T: AsyncTask + Component>(mut commands: Commands, mut query: Query<(Entity, &mut T)>) {
  for (entity, mut task) in &mut query {
    if let Some(mut commands_queue) = task.poll_once() {
      commands.append(&mut commands_queue);
      commands.entity(entity).despawn_recursive();
    }
  }
}