use crate::constants::{
  CHUNK_SIZE, ORIGIN_CHUNK_GRID_SPAWN_POINT, ORIGIN_TILE_GRID_SPAWN_POINT, ORIGIN_WORLD_SPAWN_POINT, TILE_SIZE,
};
use crate::coordinates::point::{ChunkGrid, TileGrid, World};
use crate::coordinates::{Coords, Point};
use bevy::app::{App, Plugin};
use bevy::log::debug;
use bevy::prelude::Resource;

/// A plugin that provides the [`CurrentChunk`] resource and registers it.
pub struct CurrentChunkPlugin;

impl Plugin for CurrentChunkPlugin {
  fn build(&self, app: &mut App) {
    app.insert_resource(CurrentChunk::default());
  }
}

#[derive(Resource, Debug, Clone)]
pub struct CurrentChunk {
  centre_w: Point<World>,
  coords: Coords,
}

impl CurrentChunk {
  pub const fn get_centre_world(&self) -> Point<World> {
    self.centre_w
  }

  pub const fn get_world(&self) -> Point<World> {
    self.coords.world
  }

  pub const fn get_tile_grid(&self) -> Point<TileGrid> {
    self.coords.tile_grid
  }

  pub const fn get_chunk_grid(&self) -> Point<ChunkGrid> {
    self.coords.chunk_grid
  }

  pub const fn contains(&self, tg: Point<TileGrid>) -> bool {
    tg.x >= self.coords.tile_grid.x
      && tg.x < (self.coords.tile_grid.x + CHUNK_SIZE)
      && tg.y >= self.coords.tile_grid.y
      && tg.y < (self.coords.tile_grid.y - CHUNK_SIZE)
  }

  pub fn update(&mut self, w: Point<World>) {
    let old_value = self.coords.chunk_grid;
    let cg = Point::new_chunk_grid_from_world(w);
    self.coords.world = w;
    self.coords.chunk_grid = cg;
    self.coords.tile_grid = Point::new_tile_grid_from_world(w);
    self.centre_w = Point::new_world(
      w.x + (CHUNK_SIZE * TILE_SIZE as i32 / 2),
      w.y - (CHUNK_SIZE * TILE_SIZE as i32 / 2),
    );
    debug!("Current chunk updated from {} to {}", old_value, cg);
  }
}

impl Default for CurrentChunk {
  fn default() -> Self {
    Self {
      centre_w: Point::new_world(
        ORIGIN_WORLD_SPAWN_POINT.x + (CHUNK_SIZE * TILE_SIZE as i32 / 2),
        ORIGIN_WORLD_SPAWN_POINT.y - (CHUNK_SIZE * TILE_SIZE as i32 / 2),
      ),
      coords: Coords::new(
        ORIGIN_WORLD_SPAWN_POINT,
        ORIGIN_CHUNK_GRID_SPAWN_POINT,
        ORIGIN_TILE_GRID_SPAWN_POINT,
      ),
    }
  }
}
