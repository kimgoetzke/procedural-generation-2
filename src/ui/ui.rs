use super::settings_ui::SettingsUiPlugin;
use crate::ui::diagnostics_ui::DiagnosticsUiPlugin;
use bevy::app::{App, Plugin};

pub struct UiPlugin;

impl Plugin for UiPlugin {
  fn build(&self, app: &mut App) {
    app.add_plugins((SettingsUiPlugin, DiagnosticsUiPlugin));
  }
}
