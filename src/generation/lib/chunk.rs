use crate::coords::point::World;
use crate::coords::{Coords, Point};
use crate::generation::lib::{DraftChunk, LayeredPlane};
use crate::resources::Settings;

/// A `Chunk` represents a single chunk of the world.
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
  pub fn new(draft_chunk: DraftChunk, settings: &Settings) -> Self {
    let layered_plane = LayeredPlane::new(draft_chunk.data, settings);
    Chunk {
      coords: draft_chunk.coords,
      center: draft_chunk.center,
      layered_plane,
    }
  }
}
