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
  GenerateChunks,
  SpawnEmptyEntities,
  ScheduleSpawningTiles,
  GenerateObjectData,
  ScheduleSpawningObjects,
  ScheduleWorldPruning,
  Done,
}

#[derive(Component)]
pub struct UpdateWorldComponent {
  pub created_at: u128,
  pub status: UpdateWorldStatus,
  pub wg: Point<World>,
  pub stage_1_gen_task: Option<Task<Vec<Chunk>>>,
  pub stage_2_chunks: Vec<Chunk>,
  pub stage_3_spawn_data: Vec<(Chunk, Vec<TileData>)>,
  pub stage_4_spawn_data: Vec<(Chunk, Vec<TileData>)>,
  pub stage_5_object_data: Vec<Task<Vec<ObjectData>>>,
  pub prune_world_after: bool,
}

impl UpdateWorldComponent {
  pub fn new(wg: Point<World>, task: Task<Vec<Chunk>>, prune_world_after: bool, created_at: u128) -> Self {
    Self {
      created_at,
      status: UpdateWorldStatus::GenerateChunks,
      wg,
      stage_1_gen_task: Some(task),
      stage_2_chunks: vec![],
      stage_3_spawn_data: vec![],
      stage_4_spawn_data: vec![],
      stage_5_object_data: vec![],
      prune_world_after,
    }
  }
}

impl Display for UpdateWorldComponent {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "Update world status is [{:?}] with [wg{}, prune_world_after={}]",
      self.status, self.wg, self.prune_world_after
    )
  }
}
