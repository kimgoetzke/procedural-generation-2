use crate::constants::*;
use crate::coords::Point;
use crate::coords::point::{ChunkGrid, InternalGrid};
use crate::events::{PruneWorldEvent, RefreshMetadata, RegenerateWorldEvent};
use crate::generation::lib::{Direction, TerrainType, get_cardinal_direction_points, shared};
use crate::generation::resources::{BiomeMetadata, Climate, ElevationMetadata, Metadata};
use crate::resources::{CurrentChunk, GenerationMetadataSettings, Settings};
use crate::states::AppState;
use bevy::app::{App, Plugin, Update};
use bevy::log::*;
use bevy::prelude::{EventReader, EventWriter, NextState, OnEnter, Res, ResMut};
use noise::{BasicMulti, MultiFractal, NoiseFn, Perlin};
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};
use std::hash::Hasher;
use std::hash::{DefaultHasher, Hash};
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
  let biome_perlin: BasicMulti<Perlin> = BasicMulti::new(settings.world.noise_seed)
    .set_octaves(1)
    .set_frequency(metadata_settings.biome_noise_frequency);
  let settlement_perlin: BasicMulti<Perlin> = BasicMulti::new(settings.world.noise_seed)
    .set_octaves(1)
    .set_frequency(metadata_settings.settlement_noise_frequency);
  metadata.index.clear();
  (cg.x - METADATA_GRID_APOTHEM..=cg.x + METADATA_GRID_APOTHEM).for_each(|x| {
    (cg.y - METADATA_GRID_APOTHEM..=cg.y + METADATA_GRID_APOTHEM).for_each(|y| {
      let cg = Point::new_chunk_grid(x, y);
      generate_elevation_metadata(&mut metadata, x, y, &metadata_settings);
      generate_biome_metadata(&mut metadata, &settings, &biome_perlin, cg);
      generate_connection_points(&mut metadata, &settings, cg);
      generate_settlement_metadata(&mut metadata, &settings, &settlement_perlin, cg);
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

fn generate_elevation_metadata(metadata: &mut Metadata, x: i32, y: i32, metadata_settings: &GenerationMetadataSettings) {
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
    _ => TerrainType::Shore,
  };
  let bm = BiomeMetadata::new(cg, is_rocky, rainfall as f32, max_layer as i32, climate);
  trace!("Generated: {:?}", bm);
  metadata.biome.insert(cg, bm);
}

fn generate_connection_points(metadata: &mut ResMut<Metadata>, settings: &Settings, cg: Point<ChunkGrid>) {
  let connection_points = calculate_connection_points_for_cg(settings, &cg);
  metadata.connection.insert(cg, connection_points);
}

fn calculate_connection_points_for_cg(settings: &Settings, cg: &Point<ChunkGrid>) -> Vec<Point<InternalGrid>> {
  let mut connection_points = Vec::new();
  for (direction, neighbour_cg) in get_cardinal_direction_points(&cg) {
    let hash = generate_hash(&cg, &neighbour_cg);
    let mut rng = StdRng::seed_from_u64(hash);
    let num_points = match rng.random_range(0..100) {
      0..=40 => 0,
      _ => 1,
    };
    let mut connection_points_for_edge = (0..num_points)
      .map(|_| {
        let coordinate = rng.random_range(2..CHUNK_SIZE - 2);
        match direction {
          Direction::Top => Point::new_internal_grid(coordinate, 0),
          Direction::Right => Point::new_internal_grid(CHUNK_SIZE - 1, coordinate),
          Direction::Bottom => Point::new_internal_grid(coordinate, CHUNK_SIZE - 1),
          Direction::Left => Point::new_internal_grid(0, coordinate),
          _ => panic!(
            "Unexpected intercardinal direction: [{:?}] - only cardinal directions are valid",
            direction
          ),
        }
      })
      .collect::<Vec<_>>();
    connection_points.append(&mut connection_points_for_edge);
  }

  connection_points.sort();
  connection_points.dedup();
  if connection_points.len() == 1 {
    let mut rng = StdRng::seed_from_u64(shared::calculate_seed(*cg, settings.world.noise_seed));
    loop {
      let new_point = Point::new_internal_grid(rng.random_range(1..CHUNK_SIZE - 2), rng.random_range(1..CHUNK_SIZE - 2));
      if !connection_points.contains(&new_point) {
        trace!("Added an internal connection point {:?} for chunk {}", new_point, cg);
        connection_points.push(new_point);
        break;
      }
    }
  }

  if !connection_points.is_empty() {
    trace!(
      "{} has [{}] connection points: {:?}",
      cg,
      connection_points.len(),
      connection_points
    );
  }

  connection_points
}

/// Generates a hash based on ordered chunk grid points. This leads to the same hash for the same pair of
/// chunk grids, regardless of the order in which they are passed.
fn generate_hash(reference_cg: &Point<ChunkGrid>, neighbour_cg: &Point<ChunkGrid>) -> u64 {
  let mut hasher = DefaultHasher::new();
  let (smaller_point, larger_point) = if reference_cg < neighbour_cg {
    (*reference_cg, *neighbour_cg)
  } else {
    (*neighbour_cg, *reference_cg)
  };
  format!("{:?}:{:?}", smaller_point, larger_point).hash(&mut hasher);

  hasher.finish()
}

/// Determines whether a chunk should have a settlement based on its coordinates and the settings. If the chunk is
/// considered to be settled, buildings can be generated on it.
fn generate_settlement_metadata(
  metadata: &mut ResMut<Metadata>,
  settings: &Settings,
  perlin: &BasicMulti<Perlin>,
  cg: Point<ChunkGrid>,
) {
  let settlement_probability = settings.metadata.settlement_probability;
  let noise = (perlin.get([cg.x as f64, cg.y as f64]) + 1.) / 2.;
  let is_settled = if noise >= settlement_probability { false } else { true };
  debug!(
    "Generated settlement status [{:?}] for {} because noise is {}",
    is_settled, cg, noise
  );
  metadata.settlement.insert(cg, is_settled);
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::coords::Point;
  use crate::generation::lib::Direction;

  #[test]
  fn calculate_connection_points_in_matching_pairs_1() {
    calculate_connection_points_generates_matching_pairs(Point::new_chunk_grid(1, 1))
  }

  #[test]
  fn calculate_connection_points_in_matching_pairs_2() {
    calculate_connection_points_generates_matching_pairs(Point::new_chunk_grid(0, 0))
  }

  #[test]
  fn calculate_connection_points_in_matching_pairs_for_large_numbers_1() {
    calculate_connection_points_generates_matching_pairs(Point::new_chunk_grid(50, 100))
  }

  #[test]
  fn calculate_connection_points_in_matching_pairs_for_large_numbers_2() {
    calculate_connection_points_generates_matching_pairs(Point::new_chunk_grid(-9999, 9999))
  }

  #[test]
  fn calculate_connection_points_in_matching_pairs_for_large_numbers_3() {
    calculate_connection_points_generates_matching_pairs(Point::new_chunk_grid(91933459, 89345345))
  }

  #[test]
  fn calculate_connection_points_in_matching_pairs_for_negative_numbers_1() {
    calculate_connection_points_generates_matching_pairs(Point::new_chunk_grid(-1, -1))
  }

  #[test]
  fn calculate_connection_points_in_matching_pairs_for_negative_numbers_2() {
    calculate_connection_points_generates_matching_pairs(Point::new_chunk_grid(-25, -74))
  }

  #[test]
  fn calculate_connection_points_in_matching_pairs_for_negative_numbers_3() {
    calculate_connection_points_generates_matching_pairs(Point::new_chunk_grid(-52939252, -82445308))
  }

  fn calculate_connection_points_generates_matching_pairs(cg: Point<ChunkGrid>) {
    // Generate connection points for requested point
    let settings = Settings { ..Default::default() };
    let mut connection_points_map = std::collections::HashMap::new();
    let connection_points = calculate_connection_points_for_cg(&settings, &cg);
    connection_points_map.insert(cg, connection_points);

    // Generate connection points for all neighbors
    for (_, neighbor_cg) in get_cardinal_direction_points(&cg) {
      let neighbor_points = calculate_connection_points_for_cg(&settings, &neighbor_cg);
      connection_points_map.insert(neighbor_cg, neighbor_points);
    }

    // Assert that no point has more than the permitted number of connections
    for (cg, cps) in &connection_points_map {
      assert!(cps.len() <= 8, "Neighbor {:?} has more than 8 connections: {:?}", cg, cps);
    }

    // For each direction in turn, assert that connection points are matching pairs
    for (direction, neighbor_cg) in get_cardinal_direction_points(&cg) {
      let reference_connection_points = connection_points_map
        .get(&cg)
        .expect("Failed to find connection points for current chunk grid");
      let neighbor_connection_points = connection_points_map
        .get(&neighbor_cg)
        .expect("Failed to find connection points for current chunk grid");

      for ig in reference_connection_points {
        let (expected_direction, expected_coordinate) = match direction {
          Direction::Top if ig.y == 0 => (Direction::Bottom, ig.x),
          Direction::Bottom if ig.y == CHUNK_SIZE - 1 => (Direction::Top, ig.x),
          Direction::Left if ig.x == 0 => (Direction::Right, ig.y),
          Direction::Right if ig.x == CHUNK_SIZE => (Direction::Left, ig.y),
          _ => continue,
        };
        let is_matching_pair = neighbor_connection_points
          .iter()
          .any(|neighbour_ig| match expected_direction {
            Direction::Top => neighbour_ig.y == 0 && neighbour_ig.x == expected_coordinate,
            Direction::Bottom => neighbour_ig.y == CHUNK_SIZE - 1 && neighbour_ig.x == expected_coordinate,
            Direction::Left => neighbour_ig.x == 0 && neighbour_ig.y == expected_coordinate,
            Direction::Right => neighbour_ig.x == CHUNK_SIZE - 1 && neighbour_ig.y == expected_coordinate,
            _ => false,
          });
        assert!(
          is_matching_pair,
          "No matching connection point for {:?} at {:?} in neighbor {:?}",
          direction, ig, neighbor_cg
        );
      }
    }
  }
}
