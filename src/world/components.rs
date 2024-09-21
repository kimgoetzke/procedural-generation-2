use crate::coords::Coords;
use crate::world::layered_plane::LayeredPlane;
use crate::world::tile::Tile;
use bevy::prelude::{Component, Entity};

#[derive(Component, Debug, Clone, PartialEq)]
pub struct TileComponent {
  pub tile: Tile,
  pub parent_entity: Entity,
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct ChunkComponent {
  pub coords: Coords,
  pub layered_plane: LayeredPlane,
}
