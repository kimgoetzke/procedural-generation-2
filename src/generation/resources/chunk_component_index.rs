use crate::coords::Point;
use crate::coords::point::World;
use crate::generation::lib::ChunkComponent;
use bevy::app::{App, Plugin};
use bevy::log::trace;
use bevy::platform::collections::HashMap;
use bevy::prelude::{Add, IntoSystem, Name, Observer, On, Query, Remove, ResMut, Resource};

pub struct ChunkComponentIndexPlugin;

impl Plugin for ChunkComponentIndexPlugin {
  fn build(&self, app: &mut App) {
    app.init_resource::<ChunkComponentIndex>().world_mut().spawn_batch([
      (
        Observer::new(IntoSystem::into_system(on_add_chunk_component_trigger)),
        Name::new("Observer: Add ChunkComponent"),
      ),
      (
        Observer::new(IntoSystem::into_system(on_remove_chunk_component_trigger)),
        Name::new("Observer: Remove ChunkComponent"),
      ),
    ]);
  }
}

/// Contains a clone of the [`ChunkComponent`] of each chunk entity that currently exists in the world. This index is
/// kept up-to-date by observing the [`OnAdd`] and [`OnRemove`] triggers.
#[derive(Resource, Default)]
pub struct ChunkComponentIndex {
  map: HashMap<Point<World>, ChunkComponent>,
}

impl ChunkComponentIndex {
  pub fn get(&self, w: &Point<World>) -> Option<&ChunkComponent> {
    if let Some(entity) = self.map.get(w) {
      Some(entity)
    } else {
      None
    }
  }

  pub fn size(&self) -> usize {
    self.map.len()
  }
}

fn on_add_chunk_component_trigger(
  trigger: On<Add, ChunkComponent>,
  query: Query<&ChunkComponent>,
  mut index: ResMut<ChunkComponentIndex>,
) {
  let cc = query.get(trigger.entity).expect("Failed to get ChunkComponent");
  index.map.insert(cc.coords.world, cc.clone());
  trace!("ChunkComponentIndex <- Added ChunkComponent key {:?}", cc.coords.world);
}

fn on_remove_chunk_component_trigger(
  trigger: On<Remove, ChunkComponent>,
  query: Query<&ChunkComponent>,
  mut index: ResMut<ChunkComponentIndex>,
) {
  let cc = query.get(trigger.entity).expect("Failed to get ChunkComponent");
  index.map.remove(&cc.coords.world);
  trace!("ChunkComponentIndex -> Removed ChunkComponent with key {:?}", cc.coords.world);
}
