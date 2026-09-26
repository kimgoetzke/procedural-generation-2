use crate::coordinates::Point;
use crate::coordinates::point::ChunkGrid;
use crate::generation::model::pipeline::ChunkComponent;
use bevy::platform::collections::HashMap;
use bevy::prelude::Resource;

/// Contains a clone of the [`ChunkComponent`] of each chunk entity that currently exists in the world. This index is
/// kept up-to-date by the [`crate::generation::generation_resources::ChunkComponentIndexPlugin`].
#[derive(Resource, Default)]
pub struct ChunkComponentIndex {
  map: HashMap<Point<ChunkGrid>, ChunkComponent>,
}

impl ChunkComponentIndex {
  pub fn get(&self, cg: &Point<ChunkGrid>) -> Option<&ChunkComponent> {
    self.map.get(cg)
  }

  pub fn size(&self) -> usize {
    self.map.len()
  }

  pub(in crate::generation) fn insert(&mut self, component: ChunkComponent) {
    self.map.insert(component.coords.chunk_grid, component);
  }

  pub(in crate::generation) fn remove(&mut self, cg: &Point<ChunkGrid>) {
    self.map.remove(cg);
  }
}
