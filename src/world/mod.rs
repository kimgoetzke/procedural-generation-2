use crate::world::debug::DebugPlugin;
use crate::world::object::ObjectGenerationPlugin;
use crate::world::pre_processor::PreProcessorPlugin;
use crate::world::resources::WorldResourcesPlugin;
use crate::world::world::WorldPlugin;
use bevy::app::{App, Plugin};
use bevy::prelude::*;
use std::time::SystemTime;
use tile::Tile;

mod chunk;
mod components;
mod debug;
pub(crate) mod direction;
mod draft_chunk;
mod layered_plane;
mod neighbours;
mod object;
mod plane;
mod pre_processor;
mod resources;
mod terrain_type;
mod tile;
mod tile_type;
mod world;

pub struct GenerationPlugin;

impl Plugin for GenerationPlugin {
  fn build(&self, app: &mut App) {
    app.add_plugins((
      WorldPlugin,
      WorldResourcesPlugin,
      PreProcessorPlugin,
      ObjectGenerationPlugin,
      DebugPlugin,
    ));
  }
}

#[derive(Clone, Copy)]
struct TileData {
  entity: Entity,
  parent_entity: Entity,
  tile: Tile,
}

impl TileData {
  fn new(entity: Entity, parent_entity: Entity, tile: Tile) -> Self {
    Self {
      entity,
      parent_entity,
      tile,
    }
  }
}

fn get_time() -> u128 {
  SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()
}
