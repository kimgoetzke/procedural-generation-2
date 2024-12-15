use crate::constants::*;
use crate::coords::point::ChunkGrid;
use crate::coords::Point;
use crate::generation::lib::{shared, TerrainType};
use crate::generation::resources::{BiomeMetadata, Climate, ElevationMetadata, Metadata};
use crate::resources::{CurrentChunk, Settings};
use crate::states::AppState;
use bevy::app::{App, Plugin, Update};
use bevy::log::*;
use bevy::prelude::{NextState, OnEnter, Res, ResMut};
use noise::{BasicMulti, MultiFractal, NoiseFn, Perlin};
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};
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
  let start_time = shared::get_time();
  let perlin: BasicMulti<Perlin> = BasicMulti::new(settings.world.noise_seed)
    .set_octaves(1)
    .set_frequency(settings.metadata.noise_frequency);
  let elevation_frequency = settings.metadata.elevation_frequency;
  metadata.index.clear();
  (cg.x - METADATA_GRID_APOTHEM..=cg.x + METADATA_GRID_APOTHEM).for_each(|x| {
    (cg.y - METADATA_GRID_APOTHEM..=cg.y + METADATA_GRID_APOTHEM).for_each(|y| {
      let cg = Point::new_chunk_grid(x, y);
      generate_elevation_metadata(&mut metadata, x, y, elevation_frequency);
      generate_biome_metadata(&mut metadata, &settings, &perlin, cg);
      metadata.index.push(cg);
    })
  });
  debug!(
    "Updated metadata based on current chunk {} using inputs [elevation_frequency={}, noise_frequency={}] in {} ms on {}",
    cg,
    elevation_frequency,
    settings.metadata.noise_frequency,
    shared::get_time() - start_time,
    shared::thread_name()
  );
}

fn generate_elevation_metadata(metadata: &mut ResMut<Metadata>, x: i32, y: i32, frequency: f32) {
  let grid_size = (CHUNK_SIZE - BUFFER_SIZE) as f32;
  let x_range = get_range(x, frequency);
  let y_range = get_range(y, frequency);
  let em = ElevationMetadata {
    x_step: (x_range.end - x_range.start) / grid_size,
    x_range,
    y_step: (y_range.end - y_range.start) / grid_size,
    y_range,
  };
  let cg = Point::new_chunk_grid(x, y);
  debug!("Generated elevation metadata for {}: {:?}", cg, em);
  metadata.elevation.insert(cg, em);
}

/// Returns a range based on the given coordinate and step size. The range is rounded to 3 decimal places.
fn get_range(coordinate: i32, frequency: f32) -> Range<f32> {
  let start = ((coordinate as f32 * frequency).sin() * 1000.0).round() / 1000.0;
  let end = (((coordinate + 1) as f32 * frequency).sin() * 1000.0).round() / 1000.0;

  Range { start, end }
}

fn generate_biome_metadata(
  metadata: &mut ResMut<Metadata>,
  settings: &Settings,
  perlin: &BasicMulti<Perlin>,
  cg: Point<ChunkGrid>,
) {
  let mut rng = StdRng::seed_from_u64(shared::calculate_seed(cg, settings.world.noise_seed));
  let rainfall = (perlin.get([cg.x as f64, cg.y as f64]) + 1.) / 2.;
  let climate = Climate::from(rainfall);
  let is_rocky = rng.gen_bool(METADATA_IS_ROCKY_PROBABILITY);
  let max_layer = match rainfall {
    n if n > 0.75 => TerrainType::Land3,
    n if n > 0.5 => TerrainType::Land2,
    n if n > 0.25 => TerrainType::Land1,
    _ => TerrainType::ShallowWater,
  };
  let bm = BiomeMetadata::new(cg, is_rocky, rainfall as f32, max_layer as i32, climate);
  trace!("Generated: {:?}", bm);
  metadata.biome.insert(cg, bm);
}
