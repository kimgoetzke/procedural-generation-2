use crate::constants::ELEVATION_GRID_APOTHEM;
use crate::coords::point::ChunkGrid;
use crate::coords::Point;
use crate::generation::resources::{ElevationMetadata, Metadata};
use crate::generation::{async_utils, get_time};
use crate::resources::{CurrentChunk, Settings};
use crate::states::{AppState, GenerationState};
use bevy::app::{App, Plugin};
use bevy::log::*;
use bevy::prelude::{NextState, OnEnter, Res, ResMut};
use std::ops::Range;

pub struct MetadataGeneratorPlugin;

impl Plugin for MetadataGeneratorPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(OnEnter(AppState::Initialising), generate_metadata)
      .add_systems(OnEnter(GenerationState::Generating), update_metadata);
  }
}

fn generate_metadata(
  metadata: ResMut<Metadata>,
  current_chunk: Res<CurrentChunk>,
  settings: Res<Settings>,
  mut next_state: ResMut<NextState<AppState>>,
) {
  regenerate_metadata(metadata, current_chunk.get_chunk_grid(), settings);
  debug!("Transitioning [{}] to [{:?}] state", AppState::name(), AppState::Running);
  next_state.set(AppState::Running);
}

// TODO: Fix bug where metadata update be executed after the draft chunk generation, causing a panic that could be
//  avoided by introducing another generation states.
/// Currently we're always regenerating the metadata for the entire grid. This is to allow changing the step size in
/// the UI without having visual artifacts due to already generated metadata that is then incorrect. If this becomes
/// a performance issue, we can change it but as of now, it's never taken anywhere near 1 ms.
fn update_metadata(metadata: ResMut<Metadata>, current_chunk: Res<CurrentChunk>, settings: Res<Settings>) {
  regenerate_metadata(metadata, current_chunk.get_chunk_grid(), settings);
}

fn regenerate_metadata(mut metadata: ResMut<Metadata>, cg: Point<ChunkGrid>, settings: Res<Settings>) {
  let start_time = get_time();
  let x_step = settings.metadata.elevation_step_increase_x;
  let y_step = settings.metadata.elevation_step_increase_y;
  (cg.x - ELEVATION_GRID_APOTHEM..=cg.x + ELEVATION_GRID_APOTHEM).for_each(|x| {
    (cg.y - ELEVATION_GRID_APOTHEM..=cg.y + ELEVATION_GRID_APOTHEM).for_each(|y| {
      generate_elevation_metadata(&mut metadata, x_step, y_step, x, y);
    })
  });
  debug!(
    "Updated metadata based on new current chunk using [x_step={}, y_step={}] in {} ms on {}",
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

fn get_range(coordinate: i32, elevation_step: f32) -> Range<f32> {
  Range {
    start: (((coordinate as f32 * elevation_step) - (elevation_step / 2.)) * 1000.0).round() / 1000.0,
    end: (((coordinate as f32 * elevation_step) + (elevation_step / 2.)) * 1000.0).round() / 1000.0,
  }
}
