use crate::constants::{BUFFER_SIZE, CHUNK_SIZE, CHUNK_SIZE_PLUS_BUFFER};
use crate::coords::point::{ChunkGrid, TileGrid, World};
use crate::coords::{Coords, Point};
use crate::generation::lib::debug_data::DebugData;
use crate::generation::lib::{shared, DraftTile, TerrainType};
use crate::generation::resources::Metadata;
use crate::resources::Settings;
use bevy::log::*;
use noise::{BasicMulti, MultiFractal, NoiseFn, Perlin};

/// A rather short-lived struct that is used to generate a `Chunk`. It only contains a single, flat plane of
/// `DraftTile`s. When creating a `DraftChunk`, the terrain data is generated procedurally, the result of which is
/// stored in the `data` field.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct DraftChunk {
  pub coords: Coords,
  pub center: Point<World>,
  pub data: Vec<Vec<Option<DraftTile>>>,
}

impl DraftChunk {
  /// Creates a new, flat draft chunk with terrain data based on noise by using Perlin noise.
  pub fn new(w: Point<World>, tg: Point<TileGrid>, metadata: &Metadata, settings: &Settings) -> Self {
    let coord = Coords::new_for_chunk(w, tg);
    let data = generate_terrain_data(&tg, &coord.chunk_grid, metadata, settings);
    Self {
      center: Point::new_world(tg.x + (CHUNK_SIZE_PLUS_BUFFER / 2), tg.y + (CHUNK_SIZE_PLUS_BUFFER / 2)),
      coords: coord,
      data,
    }
  }
}

/// Generates terrain data for a draft chunk based on Perlin noise. Expects `tg` to be a `Point` of type
/// `TileGrid` that describes the top-left corner of the grid.
fn generate_terrain_data(
  tg: &Point<TileGrid>,
  cg: &Point<ChunkGrid>,
  metadata: &Metadata,
  settings: &Settings,
) -> Vec<Vec<Option<DraftTile>>> {
  let start_time = shared::get_time();
  let elevation_metadata = metadata
    .elevation
    .get(cg)
    .expect(format!("Failed to get elevation metadata for {}", cg).as_str());
  let biome_metadata = metadata
    .biome
    .get(cg)
    .expect(format!("Failed to get biome metadata for {}", cg).as_str());
  let perlin: BasicMulti<Perlin> = BasicMulti::new(settings.world.noise_seed)
    .set_octaves(settings.world.noise_octaves)
    .set_frequency(settings.world.noise_frequency)
    .set_persistence(settings.world.noise_persistence);
  let amplitude = settings.world.noise_amplitude;
  let start = Point::new_tile_grid(tg.x - BUFFER_SIZE, tg.y + BUFFER_SIZE);
  let end = Point::new_tile_grid(start.x + CHUNK_SIZE_PLUS_BUFFER - 1, start.y - CHUNK_SIZE_PLUS_BUFFER + 1);
  let center = Point::new_tile_grid((start.x + end.x) / 2, (start.y + end.y) / 2);
  let max_distance = (CHUNK_SIZE_PLUS_BUFFER as f64) / 2.;
  let use_max_layer_cap = settings.general.enable_occasional_max_layer_cap;
  let falloff_strength = settings.world.falloff_strength;
  let falloff_noise_strength = settings.world.falloff_noise_strength;
  let mut tiles = vec![vec![None; CHUNK_SIZE_PLUS_BUFFER as usize]; CHUNK_SIZE_PLUS_BUFFER as usize];
  let mut ix = 0;
  let mut iy = 0;

  for ty in (end.y..=start.y).rev() {
    for tx in start.x..=end.x {
      let tg = Point::new_tile_grid(tx, ty); // Final tile grid coordinates
      let ig = Point::new_internal_grid(ix, iy); // Adjusted later when converting to tile

      // Calculate noise value
      let noise = perlin.get([tx as f64, ty as f64]);
      let clamped_noise = (noise * amplitude).clamp(-1., 1.);
      let normalised_noise = (clamped_noise + 1.) / 2.;

      // Adjust noise based on elevation metadata
      let elevation_offset = elevation_metadata.calculate_for_point(ig, CHUNK_SIZE, BUFFER_SIZE);
      let normalised_noise = (normalised_noise + elevation_offset).clamp(0., 1.);

      // Calculate falloff map value
      let falloff = calculate_terrain_falloff(
        use_max_layer_cap,
        center,
        max_distance,
        falloff_strength,
        falloff_noise_strength,
        ty,
        tx,
        clamped_noise,
      );

      // Create debug data for troubleshooting
      let debug_data = DebugData {
        noise: normalised_noise,
        noise_elevation_offset: elevation_offset,
      };

      // Determine terrain type based on noise
      let terrain = match normalised_noise {
        n if n > 0.75 => TerrainType::new_clamped(TerrainType::Forest, biome_metadata.max_layer, falloff),
        n if n > 0.6 => TerrainType::new_clamped(TerrainType::Grass, biome_metadata.max_layer, falloff),
        n if n > 0.45 => TerrainType::new_clamped(TerrainType::Sand, biome_metadata.max_layer, falloff),
        n if n > 0.3 => TerrainType::new_clamped(TerrainType::Shore, biome_metadata.max_layer, falloff),
        _ => TerrainType::Water,
      };

      let tile = DraftTile::new(ig, tg, terrain, debug_data);
      tiles[ix as usize][iy as usize] = Some(tile);
      ix += 1;
    }
    iy += 1;
    ix = 0;
  }
  trace!(
    "Generated draft chunk at {:?} in {} ms on [{}]",
    tg,
    shared::get_time() - start_time,
    shared::thread_name()
  );

  tiles
}

fn calculate_terrain_falloff(
  use_max_layer_cap: bool,
  center: Point<TileGrid>,
  max_distance: f64,
  falloff_strength: f64,
  falloff_noise_strength: f64,
  ty: i32,
  tx: i32,
  clamped_noise: f64,
) -> f64 {
  if !use_max_layer_cap {
    return 0.;
  }
  let distance_x = (tx - center.x).abs() as f64 / max_distance;
  let distance_y = (ty - center.y).abs() as f64 / max_distance;
  let distance_from_center = distance_x.max(distance_y);
  let falloff_noise = clamped_noise * falloff_noise_strength;
  let falloff = (1. - distance_from_center + falloff_noise).max(0.).powf(falloff_strength);

  falloff
}
