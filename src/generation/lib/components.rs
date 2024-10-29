use crate::coords::Coords;
use crate::generation::lib::{Chunk, LayeredPlane, Tile, TileData};
use crate::generation::object::lib::ObjectName;
use bevy::prelude::{Component, Entity};
use std::fmt::{Display, Formatter};

#[derive(Component)]
pub struct WorldComponent;

#[derive(Component, Debug, Clone, PartialEq)]
pub struct ChunkComponent {
  pub coords: Coords,
  pub layered_plane: LayeredPlane,
}

#[derive(Component, Debug, Clone, Eq, Hash, PartialEq)]
pub struct TileComponent {
  pub tile: Tile,
  pub parent_entity: Entity,
}

#[derive(Component, Debug, Clone, Eq, Hash, PartialEq)]
pub struct ObjectComponent {
  pub coords: Coords,
  pub sprite_index: usize,
  pub object_name: ObjectName,
}

#[derive(Debug)]
pub enum UpdateWorldStatus {
  SpawnEmptyEntities,
  ScheduleTileSpawning,
  GenerateObjects,
  ScheduleWorldPruning,
  Done,
}

#[derive(Component)]
pub struct UpdateWorldComponent {
  pub status: UpdateWorldStatus,
  pub stage_1_chunks: Vec<Chunk>,
  pub stage_2_spawn_data: Vec<(Chunk, Vec<TileData>)>,
  pub stage_3_spawn_data: Vec<(Chunk, Vec<TileData>)>,
  pub force_update_after: bool,
}

impl Display for UpdateWorldComponent {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "UpdateWorldComponent[status={:?}, stage_1_chunks={}, stage_2_spawn_data={}, stage_3_spawn_data={}, force_update_after={}]",
      self.status,
      self.stage_1_chunks.len(),
      self.stage_2_spawn_data.len(),
      self.stage_3_spawn_data.len(),
      self.force_update_after
    )
  }
}
