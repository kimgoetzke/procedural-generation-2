use bevy::app::{App, Plugin, Startup};
use bevy::prelude::*;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Startup, create_world);
  }
}

fn create_world(
  mut commands: Commands,
  asset_server: Res<AssetServer>,
  mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
  let texture = asset_server.load("tilesets/default.png");
  let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 9, 3, None, None);
  let texture_atlas_layout = texture_atlas_layouts.add(layout);
  commands.spawn((
    SpriteBundle {
      texture,
      transform: Transform::from_xyz(0., 0., 0.),
      ..Default::default()
    },
    TextureAtlas {
      layout: texture_atlas_layout,
      index: 25,
    },
  ));
}
