use crate::generation::model::{ChunkComponent, ChunkComponentIndex};
use bevy::app::{App, Plugin};
use bevy::log::trace;
use bevy::prelude::{Add, IntoSystem, Name, Observer, On, Query, Remove, ResMut};

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

fn on_add_chunk_component_trigger(
  trigger: On<Add, ChunkComponent>,
  query: Query<&ChunkComponent>,
  mut index: ResMut<ChunkComponentIndex>,
) {
  let cc = query.get(trigger.entity).expect("Failed to get ChunkComponent");
  index.insert(cc.clone());
  trace!("ChunkComponentIndex <- Added ChunkComponent key {:?}", cc.coords.world);
}

fn on_remove_chunk_component_trigger(
  trigger: On<Remove, ChunkComponent>,
  query: Query<&ChunkComponent>,
  mut index: ResMut<ChunkComponentIndex>,
) {
  let cc = query.get(trigger.entity).expect("Failed to get ChunkComponent");
  index.remove(&cc.coords.world);
  trace!("ChunkComponentIndex -> Removed ChunkComponent with key {:?}", cc.coords.world);
}
