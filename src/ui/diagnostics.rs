use crate::constants::*;
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
      .add_systems(Update, (update_fps_system, toggle_fps_counter_system));
  }
}

#[derive(Component)]
struct FpsUiRoot;

#[derive(Component)]
struct FpsText;

fn create_fps_counter_system(mut commands: Commands) {
  let root = commands
    .spawn((
      FpsUiRoot,
      NodeBundle {
        background_color: BackgroundColor(VERY_DARK.with_alpha(0.5)),
        z_index: ZIndex::Global(i32::MAX),
        style: Style {
          position_type: PositionType::Absolute,
          right: Val::Percent(1.),
          top: Val::Percent(1.),
          bottom: Val::Auto,
          left: Val::Auto,
          padding: UiRect::all(Val::Px(4.0)),
          ..Default::default()
        },
        ..Default::default()
      },
    ))
    .id();
  let text = commands
    .spawn((
      FpsText,
      TextBundle {
        text: Text::from_sections([
          TextSection {
            value: "FPS: ".into(),
            style: TextStyle {
              color: LIGHT,
              ..default()
            },
          },
          TextSection {
            value: " N/A".into(),
            style: TextStyle {
              color: LIGHT,
              ..default()
            },
          },
        ]),
        ..Default::default()
      },
    ))
    .id();
  commands.entity(root).push_children(&[text]);
}

fn update_fps_system(diagnostics: Res<DiagnosticsStore>, mut query: Query<&mut Text, With<FpsText>>) {
  for mut text in &mut query {
    if let Some(value) = diagnostics
      .get(&FrameTimeDiagnosticsPlugin::FPS)
      .and_then(|fps| fps.smoothed())
    {
      text.sections[1].value = format!("{value:>4.0}");
      text.sections[1].style.color = if value >= 65.0 {
        GREEN
      } else if value >= 50.0 {
        YELLOW
      } else if value >= 40.0 {
        ORANGE
      } else {
        RED
      }
    } else {
      text.sections[1].value = " N/A".into();
      text.sections[1].style.color = LIGHT;
    }
  }
}

fn toggle_fps_counter_system(
  mut fps_ui_root: Query<&mut Visibility, With<FpsUiRoot>>,
  keyboard_input: Res<ButtonInput<KeyCode>>,
) {
  if keyboard_input.just_pressed(KeyCode::F12) {
    let mut vis = fps_ui_root.single_mut();
    *vis = match *vis {
      Visibility::Hidden => Visibility::Visible,
      _ => Visibility::Hidden,
    };
  }
}
