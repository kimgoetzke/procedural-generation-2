use crate::constants::*;
use crate::events::ToggleDebugInfo;
use crate::resources::Settings;
use bevy::app::{App, Plugin, Update};
use bevy::diagnostic::DiagnosticsStore;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;

pub struct DiagnosticsUiPlugin;

impl Plugin for DiagnosticsUiPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_plugins(FrameTimeDiagnosticsPlugin::default())
      .add_systems(Startup, create_fps_counter_system)
      .add_systems(Update, (update_fps_system, toggle_fps_counter_event));
  }
}

#[derive(Component)]
struct FpsUiRoot;

#[derive(Component)]
struct FpsText;

fn create_fps_counter_system(mut commands: Commands) {
  commands
    .spawn((
      Name::new("FPS Counter"),
      FpsUiRoot,
      Node {
        position_type: PositionType::Absolute,
        right: Val::Percent(1.),
        top: Val::Percent(1.),
        bottom: Val::Auto,
        left: Val::Auto,
        padding: UiRect::all(Val::Px(4.0)),
        margin: UiRect::all(Val::Px(1.0)),
        ..Default::default()
      },
      // BackgroundColor(VERY_DARK.with_alpha(0.5)),
      Text::new("FPS: "),
      TextColor(LIGHT),
    ))
    .with_child((TextSpan::new("N/A"), FpsText, TextColor(LIGHT)));
}

fn update_fps_system(diagnostics: Res<DiagnosticsStore>, mut query: Query<&mut TextSpan, With<FpsText>>) {
  for mut span in &mut query {
    if let Some(value) = diagnostics
      .get(&FrameTimeDiagnosticsPlugin::FPS)
      .and_then(|fps| fps.smoothed())
    {
      **span = format!("{value:>4.0}");
    } else {
      **span = " N/A".into();
    }
  }
}

fn toggle_fps_counter_event(
  mut events: EventReader<ToggleDebugInfo>,
  mut fps_ui_root: Query<&mut Visibility, With<FpsUiRoot>>,
  settings: Res<Settings>,
) {
  let event_count = events.read().count();
  if event_count > 0 {
    for mut visibility in fps_ui_root.iter_mut() {
      *visibility = match settings.general.enable_tile_debugging {
        true => Visibility::Visible,
        false => Visibility::Hidden,
      };
    }
  }
}
