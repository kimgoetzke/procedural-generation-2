use crate::coords::{Coords, Point, World};
use crate::generation::draft_chunk::DraftChunk;
use crate::generation::layered_plane::LayeredPlane;
use crate::resources::Settings;
use bevy::prelude::Res;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Chunk {
  pub coords: Coords,
  pub center: Point<World>,
  pub layered_plane: LayeredPlane,
}

impl Chunk {
  /// Creates a new chunk from a draft chunk by converting the flat terrain data from the draft chunk into a
  /// `LayeredPlane`. As a result, a chunk has multiple layers of terrain data, each of which contains rich information
  /// about the `Tile`s that make up the terrain including their `TileType`s.
  pub fn new(draft_chunk: DraftChunk, settings: &Res<Settings>) -> Self {
    let layered_plane = LayeredPlane::new(draft_chunk.data, settings);
    Chunk {
      coords: draft_chunk.coords,
      center: draft_chunk.center,
      layered_plane,
    }
  }
}
