use super::settings_ui::SettingsUiPlugin;
use crate::ui::diagnostics_ui::DiagnosticsUiPlugin;
use bevy::app::{App, Plugin};

pub struct UiPlugins;

impl Plugin for UiPlugins {
  fn build(&self, app: &mut App) {
    app.add_plugins((SettingsUiPlugin, DiagnosticsUiPlugin));
  }
}
