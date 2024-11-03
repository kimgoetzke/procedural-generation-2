use crate::coords::point::{TileGrid, World};
use crate::coords::Point;
use bevy::prelude::{App, Event, Plugin};

pub struct SharedEventsPlugin;

impl Plugin for SharedEventsPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_event::<RegenerateWorldEvent>()
      .add_event::<ToggleDebugInfo>()
      .add_event::<MouseClickEvent>()
      .add_event::<UpdateWorldEvent>()
      .add_event::<PruneWorldEvent>();
  }
}

#[derive(Event)]
/// An event that triggers the regeneration of the world. It will cause the world entity and all its descendants to be
/// removed before generating an entirely new world based on the current `Settings`.
pub struct RegenerateWorldEvent {}

#[derive(Event)]
/// An event that triggers the evaluation of the world, causing the generation of new chunks and/or the despawning of
/// distant chunks (by triggering `PruneWorldEvent` at the end), if necessary. Will never remove the world entity.
pub struct UpdateWorldEvent {
  /// Will force the update to happen, even if the `CurrentChunk` has not changed. Will also suppress triggering
  /// `PruneWorldEvent` after the update which would happen by default. Used when updating the world via the UI when
  /// the `CurrentChunk` has not changed.
  pub is_forced_update: bool,
  pub w: Point<World>,
  pub tg: Point<TileGrid>,
}

#[derive(Event)]
/// An event that triggers a clean-up process of the world. In particular, this event is used to despawn all chunks
/// before generating new ones or to despawn distant chunks after having generated new chunks and changed the
/// `CurrentChunk`.
pub struct PruneWorldEvent {
  pub despawn_all_chunks: bool,
  pub update_world_after: bool,
}

#[derive(Event)]
pub struct ToggleDebugInfo {}

#[derive(Event)]
pub struct MouseClickEvent {
  pub tile_w: Point<World>,
  pub tg: Point<TileGrid>,
}
