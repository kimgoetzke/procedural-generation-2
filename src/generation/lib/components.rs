use crate::coords::point::World;
use crate::coords::{Coords, Point};
use crate::generation::lib::{Chunk, LayeredPlane, Tile, TileData};
use crate::generation::object::lib::{ObjectData, ObjectName};
use bevy::prelude::{Component, Entity};
use bevy::tasks::Task;
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
  DetermineObjects,
  SpawningObjects,
  ScheduleWorldPruning,
  Done,
}

#[derive(Component)]
pub struct UpdateWorldComponent {
  pub start: u128,
  pub status: UpdateWorldStatus,
  pub wg: Point<World>,
  pub stage_1_chunks: Vec<Chunk>,
  pub stage_2_spawn_data: Vec<(Chunk, Vec<TileData>)>,
  pub stage_3_spawn_data: Vec<(Chunk, Vec<TileData>)>,
  pub stage_4_object_data: Vec<Task<Vec<ObjectData>>>,
  pub prune_world_after: bool,
}

impl Display for UpdateWorldComponent {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "Update world status for wg{} is [{:?}] with details [1_chunks={}, 2_spawn_data={}, 3_spawn_data={}, 4_object_data={}, prune_world_after={}]",
      self.wg,
      self.status,
      self.stage_1_chunks.len(),
      self.stage_2_spawn_data.len(),
      self.stage_3_spawn_data.len(),
      self.stage_4_object_data.len(),
      self.prune_world_after
    )
  }
}
