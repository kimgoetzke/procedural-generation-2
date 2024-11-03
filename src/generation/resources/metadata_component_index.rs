use crate::coords::point::ChunkGrid;
use crate::coords::Point;
use crate::generation::lib::MetadataComponent;
use bevy::app::{App, Plugin};
use bevy::log::*;
use bevy::prelude::{OnAdd, OnRemove, Query, ResMut, Resource, Trigger};
use bevy::utils::HashMap;

pub struct MetadataComponentIndexPlugin;

impl Plugin for MetadataComponentIndexPlugin {
  fn build(&self, app: &mut App) {
    app
      .init_resource::<MetadataComponentIndex>()
      .observe(on_add_metadata_component_trigger)
      .observe(on_remove_metadata_component_trigger);
  }
}

#[derive(Resource, Default)]
pub struct MetadataComponentIndex {
  map: HashMap<Point<ChunkGrid>, MetadataComponent>,
}

impl MetadataComponentIndex {
  pub fn get(&self, cg: Point<ChunkGrid>) -> Option<&MetadataComponent> {
    if let Some(entity) = self.map.get(&cg) {
      Some(entity)
    } else {
      None
    }
  }
}

fn on_add_metadata_component_trigger(
  trigger: Trigger<OnAdd, MetadataComponent>,
  query: Query<&MetadataComponent>,
  mut index: ResMut<MetadataComponentIndex>,
) {
  let cc = query.get(trigger.entity()).expect("Failed to get MetadataComponent");
  index.map.insert(cc.cg, cc.clone());
  debug!("MetadataComponentIndex <- Added MetadataComponent key {:?}", cc.cg);
}

fn on_remove_metadata_component_trigger(
  trigger: Trigger<OnRemove, MetadataComponent>,
  query: Query<&MetadataComponent>,
  mut index: ResMut<MetadataComponentIndex>,
) {
  let cc = query.get(trigger.entity()).expect("Failed to get MetadataComponent");
  index.map.remove(&cc.cg);
  debug!("MetadataComponentIndex -> Removed MetadataComponent with key {:?}", cc.cg);
}
