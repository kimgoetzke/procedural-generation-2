use crate::coords::point::World;
use crate::coords::Point;
use crate::generation::chunk::Chunk;
use crate::generation::tile_data::TileData;
use crate::generation::world::pre_render_processor::PreRenderProcessorPlugin;
use crate::generation::world::world_generator::WorldGeneratorPlugin;
use crate::resources::Settings;
use bevy::app::{App, Plugin};
use bevy::prelude::{Commands, Entity, Res};

mod pre_render_processor;
mod world_generator;

pub struct WorldGenerationPlugin;

impl Plugin for WorldGenerationPlugin {
  fn build(&self, app: &mut App) {
    app.add_plugins((WorldGeneratorPlugin, PreRenderProcessorPlugin));
  }
}

pub fn generate_world(mut commands: &mut Commands, settings: &Res<Settings>) -> Vec<(Chunk, Vec<TileData>)> {
  world_generator::generate_world(&mut commands, settings)
}

pub fn generate_chunks(
  mut commands: &mut Commands,
  world: Entity,
  chunks_to_spawn: Vec<Point<World>>,
  settings: &Res<Settings>,
) -> Vec<(Chunk, Vec<TileData>)> {
  world_generator::generate_chunks(&mut commands, world, chunks_to_spawn, settings)
}
