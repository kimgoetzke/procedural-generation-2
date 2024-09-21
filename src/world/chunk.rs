use crate::coords::{Coords, Point};
use crate::resources::Settings;
use crate::world::draft_chunk::DraftChunk;
use crate::world::layered_plane::LayeredPlane;
use bevy::prelude::Res;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Chunk {
  pub coords: Coords,
  pub center: Point,
  pub layered_plane: LayeredPlane,
}

impl Chunk {
  pub fn new(draft_chunk: DraftChunk, settings: &Res<Settings>) -> Self {
    let layered_plane = LayeredPlane::new(draft_chunk.plane.clone(), settings);
    Chunk {
      coords: draft_chunk.coords,
      center: draft_chunk.center,
      layered_plane,
    }
  }
}

#[allow(dead_code)]
pub fn get_chunk_spawn_points(spawn_point: &Point, adjustment: i32) -> [Point; 9] {
  let p = spawn_point;
  let adjustment = adjustment - 1;
  [
    Point::new(p.x - adjustment, p.y + adjustment),
    Point::new(p.x, p.y + adjustment),
    Point::new(p.x + adjustment, p.y + adjustment),
    Point::new(p.x - adjustment, p.y),
    Point::new(p.x, p.y),
    Point::new(p.x + adjustment, p.y),
    Point::new(p.x - adjustment, p.y - adjustment),
    Point::new(p.x, p.y - adjustment),
    Point::new(p.x + adjustment, p.y - adjustment),
  ]
}
