use crate::coords::point::World;
use crate::coords::Point;
use crate::generation::lib::ChunkComponent;
use bevy::app::{App, Plugin};
use bevy::log::trace;
use bevy::prelude::{OnAdd, OnRemove, Query, ResMut, Resource, Trigger};
use bevy::utils::HashMap;

pub struct ChunkComponentIndexPlugin;

impl Plugin for ChunkComponentIndexPlugin {
  fn build(&self, app: &mut App) {
    app
      .init_resource::<ChunkComponentIndex>()
      .add_observer(on_add_chunk_component_trigger)
      .add_observer(on_remove_chunk_component_trigger);
  }
}

/// Contains a clone of the `ChunkComponent` of each chunk entity that currently exists in the world. This index is
/// kept up-to-date by observing the `OnAdd<ChunkComponent>` and `OnRemove<ChunkComponent>` triggers.
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
  trigger: Trigger<OnAdd, ChunkComponent>,
  query: Query<&ChunkComponent>,
  mut index: ResMut<ChunkComponentIndex>,
) {
  let cc = query.get(trigger.entity()).expect("Failed to get ChunkComponent");
  index.map.insert(cc.coords.world, cc.clone());
  trace!("ChunkComponentIndex <- Added ChunkComponent key {:?}", cc.coords.world);
}

fn on_remove_chunk_component_trigger(
  trigger: Trigger<OnRemove, ChunkComponent>,
  query: Query<&ChunkComponent>,
  mut index: ResMut<ChunkComponentIndex>,
) {
  let cc = query.get(trigger.entity()).expect("Failed to get ChunkComponent");
  index.map.remove(&cc.coords.world);
  trace!("ChunkComponentIndex -> Removed ChunkComponent with key {:?}", cc.coords.world);
}
