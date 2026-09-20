use crate::messages::ToggleDiagnosticsMessage;
use crate::resources::Settings;
use bevy::app::{App, Plugin};
use bevy::prelude::*;
use iyes_perf_ui::PerfUiPlugin;
use iyes_perf_ui::prelude::{
  PerfUiEntryCpuUsage, PerfUiEntryEntityCount, PerfUiEntryFPS, PerfUiEntryFPSWorst, PerfUiEntryFrameTime,
  PerfUiEntryFrameTimeWorst, PerfUiEntryMemUsage, PerfUiEntryRenderCpuTime, PerfUiEntryRenderGpuTime, PerfUiRoot,
  PerfUiWidgetBar,
};

pub struct DiagnosticsUiPlugin;

impl Plugin for DiagnosticsUiPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_plugins(PerfUiPlugin)
      .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
      .add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin::default())
      .add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin)
      .add_plugins(bevy::render::diagnostic::RenderDiagnosticsPlugin)
      .add_systems(Startup, add_perf_ui_system)
      .add_systems(Update, toggle_ui_message.before(iyes_perf_ui::PerfUiSet::Setup));
  }
}

fn add_perf_ui_system(mut commands: Commands, settings: Res<Settings>) {
  if !settings.general.display_diagnostics {
    return;
  }
  add_perf_ui(&mut commands);
}

fn toggle_ui_message(
  mut messages: MessageReader<ToggleDiagnosticsMessage>,
  q_root: Query<Entity, With<PerfUiRoot>>,
  mut commands: Commands,
  settings: Res<Settings>,
) {
  let message_count = messages.read().count();
  if message_count > 0 {
    if let Ok(e) = q_root.single() {
      if !settings.general.display_diagnostics {
        commands.entity(e).despawn();
      }
    } else if settings.general.display_diagnostics {
      add_perf_ui(&mut commands);
    }
  }
}

fn add_perf_ui(commands: &mut Commands) {
  commands.spawn((
    PerfUiWidgetBar::new(PerfUiEntryFPS::default()),
    PerfUiWidgetBar::new(PerfUiEntryFPSWorst::default()),
    PerfUiEntryFrameTime::default(),
    PerfUiEntryFrameTimeWorst::default(),
    PerfUiEntryRenderCpuTime::default(),
    PerfUiEntryRenderGpuTime::default(),
    PerfUiWidgetBar::new(PerfUiEntryEntityCount::default()),
    PerfUiWidgetBar::new(PerfUiEntryCpuUsage::default()),
    PerfUiWidgetBar::new(PerfUiEntryMemUsage::default()),
  ));
}
