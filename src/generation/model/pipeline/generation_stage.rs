use crate::generation::model::Chunk;
use crate::generation::object::model::{ObjectData, ObjectGrid};
use bevy::prelude::Entity;
use bevy::tasks::Task;
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum GenerationStage {
  /// Stage 1: Check if required metadata this
  /// [`WorldGenerationComponent`](crate::generation::model::WorldGenerationComponent) exists. If no, return current stage.
  /// Otherwise, send message to clean up not-needed chunks and schedule chunk generation and return the `Task`.
  Stage1(bool),
  /// Stage 2: Await completion of chunk generation task, then use [`crate::generation::model::ChunkComponentIndex`]
  /// to check if any of the chunks already exists. Return all [`Chunk`]s that don't exist yet, so they can be spawned.
  Stage2(Task<Vec<Chunk>>),
  /// Stage 3: If [`Chunk`]s are provided and no chunk at the "proposed" location exists, spawn the chunk(s) and return
  /// [`Chunk`]-[`Entity`] pairs. If no [`Chunk`]s provided, set [`GenerationStage`] to clean-up stage.
  Stage3(Vec<Chunk>),
  /// Stage 4: If [`Chunk`]-[`Entity`] pairs are provided and [`Entity`]s still exists, spawn tiles for each [`Chunk`]
  /// and return [`Chunk`]-[`Entity`] pairs again for further processing.
  Stage4(Vec<(Chunk, Entity)>),
  /// Stage 5: If [`Chunk`]-[`Entity`] pairs are provided and [`Entity`]s still exists, generate an [`ObjectGrid`].
  Stage5(Vec<(Chunk, Entity)>),
  /// Stage 6: If [`Chunk`]-[`Entity`]-[`ObjectGrid`] triplets are provided and [`Entity`]s still exists, schedule
  /// a task to calculate paths and update the [`ObjectGrid`]s accordingly for each of the triplet. Return the updated
  /// triplets for further processing.
  Stage6(Task<Vec<(Chunk, Entity, ObjectGrid)>>),
  /// Stage 7: If [`Chunk`]-[`Entity`]-[`ObjectGrid`] triplets are provided and [`Entity`]s still exists, schedule
  /// a task to generate buildings and other decorative objects, then convert the [`ObjectGrid`]s to
  /// [`Vec<ObjectData>`], which is used to spawn any sprites in a separate step. Return a [`Task`] for each chunk.
  ///
  /// NOTE: The [`ObjectData`] must always be generated and returned for any sprites to be spawned, even if the
  /// generation of details is disabled, because paths also require object sprites to be spawned.
  Stage7(Task<Vec<(Chunk, Entity, ObjectGrid)>>),
  /// Stage 8: If any object generation tasks is finished, schedule spawning of object sprites for the relevant chunk.
  /// If not, do nothing. Return all remaining [`Task`]s until all are finished, then proceed to next stage.
  Stage8(Vec<Task<Vec<ObjectData>>>),
  /// Stage 9: Despawn the [`WorldGenerationComponent`](crate::generation::model::WorldGenerationComponent) and, if
  /// necessary, fire a (second) message to clean up unneeded chunks.
  Stage9,
  Done,
}

impl PartialEq for GenerationStage {
  fn eq(&self, other: &Self) -> bool {
    matches!(
      (self, other),
      (GenerationStage::Stage1(_), GenerationStage::Stage1(_))
        | (GenerationStage::Stage2(_), GenerationStage::Stage2(_))
        | (GenerationStage::Stage3(_), GenerationStage::Stage3(_))
        | (GenerationStage::Stage4(_), GenerationStage::Stage4(_))
        | (GenerationStage::Stage5(_), GenerationStage::Stage5(_))
        | (GenerationStage::Stage6(_), GenerationStage::Stage6(_))
        | (GenerationStage::Stage7(_), GenerationStage::Stage7(_))
        | (GenerationStage::Stage8(_), GenerationStage::Stage8(_))
        | (GenerationStage::Stage9, GenerationStage::Stage9)
        | (GenerationStage::Done, GenerationStage::Done)
    )
  }
}

impl Display for GenerationStage {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      Self::Stage1(_) => write!(f, "Stage 1"),
      Self::Stage2(_) => write!(f, "Stage 2"),
      Self::Stage3(_) => write!(f, "Stage 3"),
      Self::Stage4(_) => write!(f, "Stage 4"),
      Self::Stage5(_) => write!(f, "Stage 5"),
      Self::Stage6(_) => write!(f, "Stage 6"),
      Self::Stage7(_) => write!(f, "Stage 7"),
      Self::Stage8(_) => write!(f, "Stage 8"),
      Self::Stage9 => write!(f, "Stage 9"),
      Self::Done => write!(f, "Done"),
    }
  }
}
