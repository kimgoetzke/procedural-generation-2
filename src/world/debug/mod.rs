use crate::world::debug::gizmos::GizmosPlugin;
use crate::world::debug::tile_debugger::TileDebuggerPlugin;
use bevy::app::{App, Plugin};

mod gizmos;
pub mod tile_debugger;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
  fn build(&self, app: &mut App) {
    app.add_plugins(TileDebuggerPlugin).add_plugins(GizmosPlugin);
  }
}
