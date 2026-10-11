use crate::generation::debug::gizmos::GizmosPlugin;
use crate::generation::debug::tile_debugger::TileDebuggerPlugin;
use bevy::app::{App, Plugin};

pub struct DebugPlugins;

impl Plugin for DebugPlugins {
  fn build(&self, app: &mut App) {
    app.add_plugins(TileDebuggerPlugin).add_plugins(GizmosPlugin);
  }
}
