use crate::world::layered_plane::LayeredPlane;
use crate::world::tile::Tile;
use bevy::prelude::Component;

#[derive(Component, Debug, Clone, PartialEq)]
pub struct TileComponent {
  pub tile: Tile,
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct ChunkComponent {
  pub layered_plane: LayeredPlane,
}
