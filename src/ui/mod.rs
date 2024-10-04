mod diagnostics;
mod settings;

use crate::ui::diagnostics::DiagnosticsUiPlugin;
use bevy::app::{App, Plugin};
use settings::SettingsUiPlugin;

pub struct UiPlugin;

impl Plugin for UiPlugin {
  fn build(&self, app: &mut App) {
    app.add_plugins((SettingsUiPlugin, DiagnosticsUiPlugin));
  }
}
