use crate::coords::Point;
use crate::coords::point::{ChunkGrid, TileGrid, World};
use bevy::prelude::{App, Message, Plugin};

/// A plugin that registers all shared messages used across multiple plugins and systems.
pub struct SharedMessagesPlugin;

impl Plugin for SharedMessagesPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_message::<RefreshMetadataMessage>()
      .add_message::<RegenerateWorldMessage>()
      .add_message::<ResetCameraMessage>()
      .add_message::<ToggleDiagnosticsMessage>()
      .add_message::<ToggleDebugInfoMessage>()
      .add_message::<MouseRightClickMessage>()
      .add_message::<UpdateWorldMessage>()
      .add_message::<PruneWorldMessage>();
  }
}

/// A message that triggers a refresh of the metadata and allows triggering either a regeneration or pruning and
/// updating of the world after.
#[derive(Message)]
pub struct RefreshMetadataMessage {
  pub regenerate_world_after: bool,
  pub prune_then_update_world_after: bool,
}

/// A message that triggers the regeneration of the world. It will cause the world entity and all its descendants to be
/// removed before generating an entirely new world based on the current [`crate::resources::Settings`].
#[derive(Message)]
pub struct RegenerateWorldMessage {}

/// A message that triggers the evaluation of the world, causing the generation of new chunks and/or the despawning of
/// distant chunks (by triggering [`PruneWorldMessage`] at the end), if necessary. Will never remove the world entity.
#[derive(Message)]
pub struct UpdateWorldMessage {
  /// Will force the update to happen, even if the [`CurrentChunk`][crate::resources::CurrentChunk] has not changed.
  /// Will also suppress triggering [`PruneWorldMessage`] after the update which would happen by default. Used when
  /// updating the world via the UI when the [`CurrentChunk`][crate::resources::CurrentChunk] has not changed.
  pub is_forced_update: bool,
  pub w: Point<World>,
  pub tg: Point<TileGrid>,
}

/// A message that triggers a clean-up process of the world. In particular, this message is used to despawn all chunks
/// before generating new ones or to despawn distant chunks after having generated new chunks and changed the
/// [`CurrentChunk`][crate::resources::CurrentChunk].
#[derive(Message)]
pub struct PruneWorldMessage {
  pub despawn_all_chunks: bool,
  pub update_world_after: bool,
}

/// A message that triggers resetting the camera to the default zoom level and position if `reset_position` is `true`.
#[derive(Message)]
pub struct ResetCameraMessage {
  pub reset_position: bool,
}

/// A message that toggles the display of diagnostics - incl. the FPS counter, CPU & memory utilisation - on or off.
#[derive(Message)]
pub struct ToggleDiagnosticsMessage {}

/// A message that toggles the display of debug information about tiles on or off. Even when this is toggled on, the
/// actual display of debug information is also dependent on whether a tile is selected for debugging.
#[derive(Message)]
pub struct ToggleDebugInfoMessage {}

/// A message that indicates that the user has right-clicked with the mouse at specific world coordinates.
#[derive(Message)]
pub struct MouseRightClickMessage {
  pub tile_w: Point<World>,
  pub cg: Point<ChunkGrid>,
  pub tg: Point<TileGrid>,
}
