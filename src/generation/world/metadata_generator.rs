use crate::constants::ELEVATION_GRID_APOTHEM;
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

fn generate_metadata(mut metadata: ResMut<Metadata>, settings: Res<Settings>, mut next_state: ResMut<NextState<AppState>>) {
  (-ELEVATION_GRID_APOTHEM..=ELEVATION_GRID_APOTHEM).for_each(|x| {
    (-ELEVATION_GRID_APOTHEM..=ELEVATION_GRID_APOTHEM).for_each(|y| {
      generate_elevation_metadata(&mut metadata, &settings, x, y);
    })
  });
  debug!("Transitioning [{}] to [{:?}] state", AppState::name(), AppState::Running);
  next_state.set(AppState::Running);
}

fn update_metadata(mut metadata: ResMut<Metadata>, current_chunk: Res<CurrentChunk>, settings: Res<Settings>) {
  let start_time = get_time();
  let cg = current_chunk.get_chunk_grid();
  (cg.x - ELEVATION_GRID_APOTHEM..=cg.x + ELEVATION_GRID_APOTHEM).for_each(|x| {
    (cg.y - ELEVATION_GRID_APOTHEM..=cg.y + ELEVATION_GRID_APOTHEM).for_each(|y| {
      if !metadata.elevation.contains_key(&Point::new_chunk_grid(x, y)) {
        generate_elevation_metadata(&mut metadata, &settings, x, y);
      }
    })
  });
  debug!(
    "Updated metadata based on new current chunk in {} ms on {}",
    get_time() - start_time,
    async_utils::thread_name()
  );
}

fn generate_elevation_metadata(metadata: &mut ResMut<Metadata>, settings: &Res<Settings>, x: i32, y: i32) {
  let elevation_step_x = settings.metadata.elevation_step_increase_x;
  let elevation_step_y = settings.metadata.elevation_step_increase_y;
  let em = ElevationMetadata {
    x: get_range(x, elevation_step_x),
    y: get_range(y, elevation_step_y),
    x_step: elevation_step_x,
    y_step: elevation_step_y,
  };
  metadata.elevation.insert(Point::new_chunk_grid(x, y), em.clone());
  trace!("Generated metadata for ({}, {}): {:?}", x, y, em);
}

fn get_range(x_or_y: i32, elevation_step: f32) -> Range<f32> {
  Range {
    start: (((x_or_y as f32 * elevation_step) - (elevation_step / 2.)) * 1000.0).round() / 1000.0,
    end: (((x_or_y as f32 * elevation_step) + (elevation_step / 2.)) * 1000.0).round() / 1000.0,
  }
}
