use crate::components::AnimationComponent;
use bevy::app::{App, Plugin};
use bevy::prelude::{Query, Res, TextureAtlas, Time, Update};

pub struct AnimationsPlugin;

impl Plugin for AnimationsPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Update, sprite_animation_system);
  }
}

fn sprite_animation_system(time: Res<Time>, mut query: Query<(&mut AnimationComponent, &mut TextureAtlas)>) {
  for (mut ac, mut atlas) in &mut query {
    ac.timer.tick(time.delta());
    if ac.timer.just_finished() {
      atlas.index = if atlas.index >= ac.index_last {
        ac.index_first
      } else {
        atlas.index + 1
      };
    }
  }
}