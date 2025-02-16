use crate::coords::point::{ChunkGrid, World};
use crate::coords::{Coords, Point};
use crate::generation::lib::{Chunk, LayeredPlane, Tile, TileData};
use crate::generation::object::lib::{ObjectData, ObjectName};
use bevy::prelude::{Component, Entity};
use bevy::tasks::Task;

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
  pub layer: i32,
}

#[derive(Debug)]
pub enum GenerationStage {
  Stage1,
  Stage2,
  Stage3,
  Stage4,
  Stage5,
  Stage6,
  Stage7,
}

/// The core component for the world generation process. Used by the world generation system. It is spawned to initiate
/// process and is removed when the process is complete.
#[derive(Component, Debug)]
pub struct WorldGenerationComponent {
  pub created_at: u128,
  pub stage: GenerationStage,
  pub w: Point<World>,
  pub cg: Point<ChunkGrid>,
  pub suppress_pruning_world: bool,
  pub stage_0_metadata: bool,
  pub stage_1_gen_task: Option<Task<Vec<Chunk>>>,
  pub stage_2_chunks: Vec<Chunk>,
  pub stage_3_spawn_data: Vec<(Chunk, Entity)>,
  pub stage_4_spawn_data: Vec<(Chunk, Entity)>,
  pub stage_5_object_data: Vec<Task<Vec<ObjectData>>>,
}

impl WorldGenerationComponent {
  pub fn new(w: Point<World>, cg: Point<ChunkGrid>, suppress_pruning_world: bool, created_at: u128) -> Self {
    Self {
      created_at,
      stage: GenerationStage::Stage1,
      w,
      cg,
      suppress_pruning_world,
      stage_0_metadata: false,
      stage_1_gen_task: None,
      stage_2_chunks: vec![],
      stage_3_spawn_data: vec![],
      stage_4_spawn_data: vec![],
      stage_5_object_data: vec![],
    }
  }
}
