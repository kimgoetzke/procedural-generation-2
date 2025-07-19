use crate::constants::*;
use crate::coords::Point;
use crate::coords::point::ChunkGrid;
use crate::events::{PruneWorldEvent, RefreshMetadata, RegenerateWorldEvent};
use crate::generation::lib::{TerrainType, shared};
use crate::generation::resources::{BiomeMetadata, Climate, ElevationMetadata, Metadata};
use crate::resources::{CurrentChunk, GenerationMetadataSettings, Settings};
use crate::states::AppState;
use bevy::app::{App, Plugin, Update};
use bevy::log::*;
use bevy::prelude::{EventReader, EventWriter, NextState, OnEnter, Res, ResMut};
use noise::{BasicMulti, MultiFractal, NoiseFn, Perlin};
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};
use std::ops::Range;

pub struct MetadataGeneratorPlugin;

impl Plugin for MetadataGeneratorPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(OnEnter(AppState::Initialising), initialise_metadata_system)
      .add_systems(Update, (update_metadata_system, refresh_metadata_event));
  }
}

/// This function is intended to be used to generate performance intensive metadata for the world prior to running the
/// main loop.
fn initialise_metadata_system(
  metadata: ResMut<Metadata>,
  current_chunk: Res<CurrentChunk>,
  settings: Res<Settings>,
  mut next_state: ResMut<NextState<AppState>>,
) {
  regenerate_metadata(metadata, current_chunk.get_chunk_grid(), &settings);
  next_state.set(AppState::Running);
}

/// Currently we're always regenerating the metadata for the entire grid. This is to allow changing the step size in
/// the UI without having visual artifacts due to already generated metadata that is then incorrect. If this becomes
/// a performance issue, we can change it but as of now, it's never taken anywhere near 1 ms.
fn update_metadata_system(mut metadata: ResMut<Metadata>, current_chunk: Res<CurrentChunk>, settings: Res<Settings>) {
  if metadata.current_chunk_cg == current_chunk.get_chunk_grid() {
    return;
  }
  metadata.current_chunk_cg = current_chunk.get_chunk_grid();
  regenerate_metadata(metadata, current_chunk.get_chunk_grid(), &settings);
}

/// Refreshes the metadata based on the current chunk and settings. Used when manually triggering a world regeneration
/// via the UI or using a keyboard shortcut. Triggers the action intended to be invoked by the user once the metadata
/// has been refreshed.
fn refresh_metadata_event(
  metadata: ResMut<Metadata>,
  current_chunk: Res<CurrentChunk>,
  settings: Res<Settings>,
  mut refresh_metadata_event: EventReader<RefreshMetadata>,
  mut regenerate_world_event: EventWriter<RegenerateWorldEvent>,
  mut prune_world_event: EventWriter<PruneWorldEvent>,
) {
  if let Some(event) = refresh_metadata_event.read().last() {
    regenerate_metadata(metadata, current_chunk.get_chunk_grid(), &settings);
    if event.regenerate_world_after {
      regenerate_world_event.write(RegenerateWorldEvent {});
    } else if event.prune_then_update_world_after && settings.general.enable_world_pruning {
      prune_world_event.write(PruneWorldEvent {
        despawn_all_chunks: true,
        update_world_after: true,
      });
    }
  }
}

fn regenerate_metadata(mut metadata: ResMut<Metadata>, cg: Point<ChunkGrid>, settings: &Settings) {
  let start_time = shared::get_time();
  let metadata_settings = settings.metadata;
  let perlin: BasicMulti<Perlin> = BasicMulti::new(settings.world.noise_seed)
    .set_octaves(1)
    .set_frequency(metadata_settings.biome_noise_frequency);
  metadata.index.clear();
  (cg.x - METADATA_GRID_APOTHEM..=cg.x + METADATA_GRID_APOTHEM).for_each(|x| {
    (cg.y - METADATA_GRID_APOTHEM..=cg.y + METADATA_GRID_APOTHEM).for_each(|y| {
      let cg = Point::new_chunk_grid(x, y);
      generate_elevation_metadata(&mut metadata, x, y, &metadata_settings);
      generate_biome_metadata(&mut metadata, &settings, &perlin, cg);
      metadata.index.push(cg);
    })
  });
  debug!(
    "Updated metadata based on current chunk {} in {} ms on {}",
    cg,
    shared::get_time() - start_time,
    shared::thread_name()
  );
}

fn generate_elevation_metadata(
  metadata: &mut ResMut<Metadata>,
  x: i32,
  y: i32,
  metadata_settings: &GenerationMetadataSettings,
) {
  let grid_size = (CHUNK_SIZE as f32 - 1.) as f64;
  let (x_range, x_step) = calculate_range_and_step_size(x, grid_size, metadata_settings);
  let (y_range, y_step) = calculate_range_and_step_size(y, grid_size, metadata_settings);
  let em = ElevationMetadata {
    is_enabled: !y_range.start.is_nan() || !y_range.end.is_nan() || !x_range.start.is_nan() || !x_range.end.is_nan(),
    x_step,
    x_range,
    y_step,
    y_range,
  };
  let cg = Point::new_chunk_grid(x, y);
  trace!("Generated elevation metadata for {}: {}", cg, em);
  metadata.elevation.insert(cg, em);
}

// TODO: Consider improving this range calculation because it's too easy for a user to "break" it via the UI
/// Returns a range and the step size for the given coordinate. The range expresses the maximum and minimum values for
/// the elevation offset. The step size is the amount of elevation change per [`crate::generation::lib::Tile`]
/// (not per [`crate::generation::lib::Chunk`]).
fn calculate_range_and_step_size(
  coordinate: i32,
  grid_size: f64,
  metadata_settings: &GenerationMetadataSettings,
) -> (Range<f64>, f64) {
  let chunk_step_size = metadata_settings.elevation_chunk_step_size;
  let offset = metadata_settings.elevation_offset;
  let frequency = 2. / chunk_step_size;
  let normalised_mod = (modulo(coordinate as f64, frequency)) / frequency;
  let is_rising = normalised_mod <= 0.5;
  let base = if is_rising {
    2.0 * normalised_mod - offset
  } else {
    (2.0 * (1.0 - normalised_mod)) - chunk_step_size - offset
  };
  let start = ((base * 10000.).round()) / 10000.;
  let mut end = (((base + chunk_step_size) * 10000.).round()) / 10000.;
  end = if end > (1. - offset) {
    (((base - chunk_step_size) * 10000.).round()) / 10000.
  } else {
    end
  };
  if is_rising {
    (Range { start, end }, step_size(start, end, grid_size, is_rising))
  } else {
    (Range { start: end, end: start }, step_size(start, end, grid_size, is_rising))
  }
}

fn modulo(a: f64, b: f64) -> f64 {
  ((a % b) + b) % b
}

fn step_size(range_start: f64, range_end: f64, grid_size: f64, is_positive: bool) -> f64 {
  let modifier = if is_positive { 1.0 } else { -1.0 };
  ((range_end - range_start) / grid_size) * modifier
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
  let is_rocky = rng.random_bool(BIOME_IS_ROCKY_PROBABILITY);
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
