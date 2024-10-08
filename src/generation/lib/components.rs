use crate::coords::Coords;
use crate::generation::lib::{LayeredPlane, Tile};
use bevy::prelude::{Component, Entity};

#[derive(Component)]
pub struct WorldComponent;

#[derive(Component, Debug, Clone, PartialEq)]
pub struct ChunkComponent {
  pub coords: Coords,
  pub layered_plane: LayeredPlane,
}

#[derive(Component, Debug, Clone, Eq, Hash, PartialEq)]
pub struct TileComponent {
  pub tile: Tile,
  pub parent_entity: Entity,
}

#[derive(Component)]
pub struct ObjectComponent {}
