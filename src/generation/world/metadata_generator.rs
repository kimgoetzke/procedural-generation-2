use crate::constants::*;
use crate::coords::point::ChunkGrid;
use crate::coords::Point;
use crate::generation::lib::{shared, TerrainType};
use crate::generation::resources::{Biome, BiomeMetadata, ElevationMetadata, Metadata};
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
  let x_step = settings.metadata.elevation_step_increase_x;
  let y_step = settings.metadata.elevation_step_increase_y;
  metadata.index.clear();
  (cg.x - METADATA_GRID_APOTHEM..=cg.x + METADATA_GRID_APOTHEM).for_each(|x| {
    (cg.y - METADATA_GRID_APOTHEM..=cg.y + METADATA_GRID_APOTHEM).for_each(|y| {
      let cg = Point::new_chunk_grid(x, y);
      generate_elevation_metadata(&mut metadata, x_step, y_step, x, y);
      generate_biome_metadata(&mut metadata, &settings, &perlin, cg);
      metadata.index.push(cg);
    })
  });
  debug!(
    "Updated metadata based on current chunk {} using inputs [x_step={}, y_step={}] in {} ms on {}",
    cg,
    x_step,
    y_step,
    shared::get_time() - start_time,
    shared::thread_name()
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
  trace!("Generated metadata for {}: {:?}", cg, em);
  metadata.elevation.insert(cg, em);
}

/// Returns a range based on the given coordinate and step size. The range is rounded to 3 decimal places.
fn get_range(coordinate: i32, elevation_step: f32) -> Range<f32> {
  Range {
    start: (((coordinate as f32 * elevation_step) - (elevation_step / 2.)) * 1000.0).round() / 1000.0,
    end: (((coordinate as f32 * elevation_step) + (elevation_step / 2.)) * 1000.0).round() / 1000.0,
  }
}

fn generate_biome_metadata(
  metadata: &mut ResMut<Metadata>,
  settings: &Settings,
  perlin: &BasicMulti<Perlin>,
  cg: Point<ChunkGrid>,
) {
  let mut rng = StdRng::seed_from_u64(shared::calculate_seed(cg, settings.world.noise_seed));
  let humidity = (perlin.get([cg.x as f64, cg.y as f64]) + 1.) / 2.;
  let biome = Biome::from(humidity);
  let is_rocky = rng.gen_bool(METADATA_IS_ROCKY_PROBABILITY);
  let max_layer = match humidity {
    n if n > 0.75 => TerrainType::Land3,
    n if n > 0.5 => TerrainType::Land2,
    n if n > 0.25 => TerrainType::Land1,
    _ => TerrainType::ShallowWater,
  };
  let bm = BiomeMetadata::new(is_rocky, humidity as f32, max_layer as i32, biome);
  debug!("Generated metadata for {}: {:?}", cg, bm);
  metadata.biome.insert(cg, bm);
}
