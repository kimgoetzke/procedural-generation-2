mod object_generator;

use crate::generation::chunk::Chunk;
use crate::generation::resources::AssetPacks;
use crate::generation::tile_data::TileData;
use crate::resources::Settings;
use bevy::app::{App, Plugin};
use bevy::prelude::{Commands, Res};

pub struct ObjectGenerationPlugin;

impl Plugin for ObjectGenerationPlugin {
  fn build(&self, _app: &mut App) {}
}

pub fn generate_objects(
  commands: &mut Commands,
  spawn_data: &mut Vec<(Chunk, Vec<TileData>)>,
  asset_packs: &Res<AssetPacks>,
  settings: &Res<Settings>,
) {
  object_generator::generate(commands, spawn_data, asset_packs, settings);
}
