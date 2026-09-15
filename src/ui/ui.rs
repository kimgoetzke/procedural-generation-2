use super::settings::SettingsUiPlugin;
use crate::ui::diagnostics::DiagnosticsUiPlugin;
use bevy::app::{App, Plugin};

pub struct UiPlugin;

impl Plugin for UiPlugin {
  fn build(&self, app: &mut App) {
    app.add_plugins((SettingsUiPlugin, DiagnosticsUiPlugin));
  }
}
