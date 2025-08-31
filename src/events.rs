use crate::coords::Point;
use crate::coords::point::{ChunkGrid, TileGrid, World};
use bevy::prelude::{App, Event, Plugin};

pub struct SharedEventsPlugin;

impl Plugin for SharedEventsPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_event::<RefreshMetadata>()
      .add_event::<RegenerateWorldEvent>()
      .add_event::<ResetCameraEvent>()
      .add_event::<ToggleDebugInfo>()
      .add_event::<MouseClickEvent>()
      .add_event::<UpdateWorldEvent>()
      .add_event::<PruneWorldEvent>();
  }
}

#[derive(Event)]
/// An event that triggers a refresh of the metadata and allows triggering either a regeneration or pruning and
/// updating of the world after.
pub struct RefreshMetadata {
  pub regenerate_world_after: bool,
  pub prune_then_update_world_after: bool,
}

#[derive(Event)]
/// An event that triggers the regeneration of the world. It will cause the world entity and all its descendants to be
/// removed before generating an entirely new world based on the current [`crate::resources::Settings`].
pub struct RegenerateWorldEvent {}

#[derive(Event)]
/// An event that triggers the evaluation of the world, causing the generation of new chunks and/or the despawning of
/// distant chunks (by triggering [`PruneWorldEvent`] at the end), if necessary. Will never remove the world entity.
pub struct UpdateWorldEvent {
  /// Will force the update to happen, even if the [`CurrentChunk`][crate::resources::CurrentChunk] has not changed.
  /// Will also suppress triggering [`PruneWorldEvent`] after the update which would happen by default. Used when
  /// updating the world via the UI when the [`CurrentChunk`][crate::resources::CurrentChunk] has not changed.
  pub is_forced_update: bool,
  pub w: Point<World>,
  pub tg: Point<TileGrid>,
}

#[derive(Event)]
/// An event that triggers a clean-up process of the world. In particular, this event is used to despawn all chunks
/// before generating new ones or to despawn distant chunks after having generated new chunks and changed the
/// [`CurrentChunk`][crate::resources::CurrentChunk].
pub struct PruneWorldEvent {
  pub despawn_all_chunks: bool,
  pub update_world_after: bool,
}

/// An event that triggers resetting the camera to the default position and zoom level.
#[derive(Event)]
pub struct ResetCameraEvent {}

#[derive(Event)]
pub struct ToggleDebugInfo {}

#[derive(Event)]
pub struct MouseClickEvent {
  pub tile_w: Point<World>,
  pub cg: Point<ChunkGrid>,
  pub tg: Point<TileGrid>,
}
