use crate::coords::point::{ChunkGrid, World};
use crate::coords::{Coords, Point};
use crate::generation::lib::{Chunk, LayeredPlane, Tile};
use crate::generation::object::lib::{ObjectData, ObjectName};
use bevy::prelude::{Component, Entity};
use bevy::tasks::Task;
use std::fmt;
use std::fmt::{Display, Formatter};

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
  /// Stage 1: Check if required metadata this `WorldGenerationComponent` exists. If no, return current stage.
  /// Otherwise, send event to clean up not-needed chunks and schedule chunk generation and return the `Task`.
  Stage1(bool),
  /// Stage 2: Await completion of chunk generation task, then use `ChunkComponentIndex` to check if any of the chunks
  /// already exists. Return all `Chunk`s that don't exist yet, so they can be spawned.
  /// Stage 3: If `Chunk`s are provided and no chunk at "proposed" location exists, spawn the chunk(s) and return
  Stage2(Task<Vec<Chunk>>),
  /// `Chunk`-`Entity` pairs. If no `Chunk`s provided, set `GenerationStage` to clean-up stage.
  Stage3(Vec<Chunk>),
  /// Stage 4: If `Chunk`-`Entity` pairs are provided and `Entity`s still exists, schedule tile spawning tasks and
  /// return `Chunk`-`Entity` pairs again.
  Stage4(Vec<(Chunk, Entity)>),
  /// Stage 5: If `Chunk`-`Entity` pairs are provided and `Entity`s still exists, schedule tasks to generate object
  /// data and return the `Task`s.
  Stage5(Vec<(Chunk, Entity)>),
  /// Stage 6: If any object generation tasks is finished, schedule spawning of object sprites for the relevant chunk.
  /// If not, do nothing. Return all remaining `Task`s until all are finished, then proceed to next stage.
  Stage6(Vec<Task<Vec<ObjectData>>>),
  /// Stage 7: Despawn the `WorldGenerationComponent`.
  Stage7,
  Done,
}

impl PartialEq for GenerationStage {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (GenerationStage::Stage1(_), GenerationStage::Stage1(_)) => true,
      (GenerationStage::Stage2(_), GenerationStage::Stage2(_)) => true,
      (GenerationStage::Stage3(_), GenerationStage::Stage3(_)) => true,
      (GenerationStage::Stage4(_), GenerationStage::Stage4(_)) => true,
      (GenerationStage::Stage5(_), GenerationStage::Stage5(_)) => true,
      (GenerationStage::Stage6(_), GenerationStage::Stage6(_)) => true,
      (GenerationStage::Stage7, GenerationStage::Stage7) => true,
      (GenerationStage::Done, GenerationStage::Done) => true,
      _ => false,
    }
  }
}

impl Display for GenerationStage {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      GenerationStage::Stage1(_) => write!(f, "Stage 1"),
      GenerationStage::Stage2(_) => write!(f, "Stage 2"),
      GenerationStage::Stage3(_) => write!(f, "Stage 3"),
      GenerationStage::Stage4(_) => write!(f, "Stage 4"),
      GenerationStage::Stage5(_) => write!(f, "Stage 5"),
      GenerationStage::Stage6(_) => write!(f, "Stage 6"),
      GenerationStage::Stage7 => write!(f, "Stage 7"),
      GenerationStage::Done => write!(f, "Done"),
    }
  }
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
}

impl WorldGenerationComponent {
  pub fn new(w: Point<World>, cg: Point<ChunkGrid>, suppress_pruning_world: bool, created_at: u128) -> Self {
    Self {
      created_at,
      stage: GenerationStage::Stage1(false),
      w,
      cg,
      suppress_pruning_world,
    }
  }
}
