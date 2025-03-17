use crate::events::ToggleDebugInfo;
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
      .add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin)
      .add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin)
      .add_plugins(bevy::render::diagnostic::RenderDiagnosticsPlugin)
      .add_systems(Startup, add_perf_ui)
      .add_systems(Update, toggle_ui.before(iyes_perf_ui::PerfUiSet::Setup));
  }
}

fn add_perf_ui(mut commands: Commands) {
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

fn toggle_ui(mut events: EventReader<ToggleDebugInfo>, q_root: Query<Entity, With<PerfUiRoot>>, mut commands: Commands) {
  let event_count = events.read().count();
  if event_count > 0 {
    if let Ok(e) = q_root.get_single() {
      commands.entity(e).despawn_recursive();
    } else {
      add_perf_ui(commands);
    }
  }
}
