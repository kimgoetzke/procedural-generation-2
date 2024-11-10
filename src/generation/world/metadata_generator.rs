use crate::constants::ELEVATION_GRID_APOTHEM;
use crate::coords::point::ChunkGrid;
use crate::coords::Point;
use crate::generation::resources::{ElevationMetadata, Metadata};
use crate::generation::{async_utils, get_time};
use crate::resources::{CurrentChunk, Settings};
use crate::states::AppState;
use bevy::app::{App, Plugin, Update};
use bevy::log::*;
use bevy::prelude::{NextState, OnEnter, Res, ResMut};
use std::ops::Range;

pub struct MetadataGeneratorPlugin;

impl Plugin for MetadataGeneratorPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(OnEnter(AppState::Initialising), initialise_metadata)
      .add_systems(Update, update_metadata);
  }
}

/// This function is intended to be used to generate performance intensive metadata for the world prior to running the
/// main loop.
fn initialise_metadata(
  metadata: ResMut<Metadata>,
  current_chunk: Res<CurrentChunk>,
  settings: Res<Settings>,
  mut next_state: ResMut<NextState<AppState>>,
) {
  regenerate_metadata(metadata, current_chunk.get_chunk_grid(), settings);
  next_state.set(AppState::Running);
}

/// Currently we're always regenerating the metadata for the entire grid. This is to allow changing the step size in
/// the UI without having visual artifacts due to already generated metadata that is then incorrect. If this becomes
/// a performance issue, we can change it but as of now, it's never taken anywhere near 1 ms.
fn update_metadata(mut metadata: ResMut<Metadata>, current_chunk: Res<CurrentChunk>, settings: Res<Settings>) {
  if metadata.current_chunk_cg == current_chunk.get_chunk_grid() {
    return;
  }
  metadata.current_chunk_cg = current_chunk.get_chunk_grid();
  regenerate_metadata(metadata, current_chunk.get_chunk_grid(), settings);
}

fn regenerate_metadata(mut metadata: ResMut<Metadata>, cg: Point<ChunkGrid>, settings: Res<Settings>) {
  let start_time = get_time();
  let x_step = settings.metadata.elevation_step_increase_x;
  let y_step = settings.metadata.elevation_step_increase_y;
  metadata.index.clear();
  (cg.x - ELEVATION_GRID_APOTHEM..=cg.x + ELEVATION_GRID_APOTHEM).for_each(|x| {
    (cg.y - ELEVATION_GRID_APOTHEM..=cg.y + ELEVATION_GRID_APOTHEM).for_each(|y| {
      let cg = Point::new_chunk_grid(x, y);
      generate_elevation_metadata(&mut metadata, x_step, y_step, x, y);
      metadata.index.push(cg);
    })
  });
  debug!(
    "Updated metadata based on current chunk {} using inputs [x_step={}, y_step={}] in {} ms on {}",
    cg,
    x_step,
    y_step,
    get_time() - start_time,
    async_utils::thread_name()
  );
}

fn generate_elevation_metadata(metadata: &mut ResMut<Metadata>, x_step: f32, y_step: f32, x: i32, y: i32) {
  let em = ElevationMetadata {
    x: get_range(x, x_step),
    y: get_range(y, y_step),
    x_step,
    y_step,
  };
  let cg = Point::new_chunk_grid(x, y);
  metadata.elevation.insert(cg, em.clone());
  trace!("Generated metadata for {}: {:?}", cg, em);
}

/// Returns a range based on the given coordinate and step size. The range is rounded to 3 decimal places.
fn get_range(coordinate: i32, elevation_step: f32) -> Range<f32> {
  Range {
    start: (((coordinate as f32 * elevation_step) - (elevation_step / 2.)) * 1000.0).round() / 1000.0,
    end: (((coordinate as f32 * elevation_step) + (elevation_step / 2.)) * 1000.0).round() / 1000.0,
  }
}
