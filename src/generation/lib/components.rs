use crate::coords::point::{ChunkGrid, World};
use crate::coords::{Coords, Point};
use crate::generation::lib::{Chunk, LayeredPlane, Tile, TileData};
use crate::generation::object::lib::{ObjectData, ObjectName};
use bevy::prelude::{Component, Entity};
use bevy::tasks::Task;
use std::fmt::{Display, Formatter};

/// A component that exists separate to the world and contains metadata about the world. This data influences the world
/// generation process in various ways but does not represent any physical entity in the world.
#[derive(Component, Debug, Clone, PartialEq)]
pub struct MetadataComponent {
  pub cg: Point<ChunkGrid>,
}

/// A simple tag component for the world entity. Used to identify the world entity in the ECS for
/// easy removal (used when regenerating the world).
#[derive(Component)]
pub struct WorldComponent;

/// A component that is attached to every chunk entity that is spawned in the world. Used in the `ChunkComponentIndex`
/// but also by other core processes such as pruning the world.
#[derive(Component, Debug, Clone, PartialEq)]
pub struct ChunkComponent {
  pub coords: Coords,
  pub layered_plane: LayeredPlane,
}

/// A component that is attached to every tile sprite that is spawned in the world. Contains the tile data
/// and the parent entity that the tile is attached to. There's a `TileComponent` for every terrain layer.
#[derive(Component, Debug, Clone, Eq, Hash, PartialEq)]
pub struct TileComponent {
  pub tile: Tile,
  pub parent_entity: Entity,
}

/// A component that is attached to every object sprite that is spawned in the world. Use for, for example,
/// debugging purposes.
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

/// The core component for the world generation process. Used by the world generation system and stores
/// the current state of the generation process. Is spawned to initiate the process and removed when the
/// process is complete.
#[derive(Component)]
pub struct UpdateWorldComponent {
  pub created_at: u128,
  pub status: UpdateWorldStatus,
  pub w: Point<World>,
  pub stage_1_gen_task: Option<Task<Vec<Chunk>>>,
  pub stage_2_chunks: Vec<Chunk>,
  pub stage_3_spawn_data: Vec<(Chunk, Vec<TileData>)>,
  pub stage_4_spawn_data: Vec<(Chunk, Vec<TileData>)>,
  pub stage_5_object_data: Vec<Task<Vec<ObjectData>>>,
  pub prune_world_after: bool,
}

impl UpdateWorldComponent {
  pub fn new(w: Point<World>, task: Task<Vec<Chunk>>, prune_world_after: bool, created_at: u128) -> Self {
    Self {
      created_at,
      status: UpdateWorldStatus::GenerateChunks,
      w,
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
      "Update world status is [{:?}] with [{}, prune_world_after={}]",
      self.status, self.w, self.prune_world_after
    )
  }
}
