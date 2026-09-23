use crate::coordinates::point::{ChunkGrid, TileGrid};
use crate::coordinates::{Coords, Point};
use crate::generation::model::{GenerationStage, LayeredPlane, Tile};
use crate::generation::object::model::ObjectName;
use bevy::prelude::{Component, Entity};

/// A simple tag component for the world entity. Used to identify the world entity in the ECS for
/// easy removal (used when regenerating the world).
#[derive(Component)]
pub struct WorldComponent;

/// A component that is attached to every chunk entity that is spawned in the world. Used in the
/// [`crate::generation::model::ChunkComponentIndex`] but also by other core processes such as pruning the world.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct ChunkComponent {
  pub coords: Coords,
  pub layered_plane: LayeredPlane,
}

/// A component that is attached to every tile layer mesh that is spawned in the world. Contains the tile data
/// and the parent entity which is a chunk. There's a [`TileMeshComponent`] for every terrain layer and even two if
/// the tiles for that layer can be both animated or not (one component for each).
#[derive(Component, Debug, Clone, Eq, Hash, PartialEq)]
pub struct TileMeshComponent {
  parent_chunk_entity: Entity,
  cg: Point<ChunkGrid>,
  tiles: Vec<Tile>,
}

impl TileMeshComponent {
  pub const fn new(parent_chunk_entity: Entity, cg: Point<ChunkGrid>, tiles: Vec<Tile>) -> Self {
    Self {
      parent_chunk_entity,
      cg,
      tiles,
    }
  }

  pub const fn cg(&self) -> Point<ChunkGrid> {
    self.cg
  }

  pub fn find_all(&self, tg: &Point<TileGrid>) -> Vec<&Tile> {
    self.tiles.iter().filter(|t| t.coords.tile_grid == *tg).collect()
  }
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

/// The core component for the world generation process. Used by the world generation system. It is spawned to initiate
/// process and is removed when the process is complete.
#[derive(Component, Debug)]
pub struct WorldGenerationComponent {
  pub created_at: u128,
  pub stage: GenerationStage,
  pub cg: Point<ChunkGrid>,
  pub suppress_pruning_world: bool,
}

impl WorldGenerationComponent {
  pub const fn new(cg: Point<ChunkGrid>, suppress_pruning_world: bool, created_at: u128) -> Self {
    Self {
      created_at,
      stage: GenerationStage::Stage1(false),
      cg,
      suppress_pruning_world,
    }
  }
}
