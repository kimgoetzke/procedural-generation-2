use crate::constants::{BUFFER_SIZE, CHUNK_SIZE, CHUNK_SIZE_PLUS_BUFFER};
use crate::coords::point::{ChunkGrid, TileGrid, World};
use crate::coords::{Coords, Point};
use crate::generation::lib::debug_data::DebugData;
use crate::generation::lib::{DraftTile, TerrainType};
use crate::generation::resources::Metadata;
use crate::generation::{async_utils, get_time};
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
  let start_time = get_time();
  let elevation_metadata = metadata
    .elevation
    .get(cg)
    .expect(format!("Failed to get elevation metadata for {}", cg).as_str());
  let perlin: BasicMulti<Perlin> = BasicMulti::new(settings.world.noise_seed)
    .set_octaves(settings.world.noise_octaves)
    .set_frequency(settings.world.noise_frequency)
    .set_persistence(settings.world.noise_persistence);
  let amplitude = settings.world.noise_amplitude;
  let start = Point::new_tile_grid(tg.x - BUFFER_SIZE, tg.y + BUFFER_SIZE);
  let end = Point::new_tile_grid(start.x + CHUNK_SIZE_PLUS_BUFFER - 1, start.y - CHUNK_SIZE_PLUS_BUFFER + 1);
  let center = Point::new_tile_grid((start.x + end.x) / 2, (start.y + end.y) / 2);
  let max_distance = (CHUNK_SIZE_PLUS_BUFFER as f64) / 2.;
  let base_elevation = settings.world.elevation;
  let falloff_strength = settings.world.falloff_strength;
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
      let normalised_noise = (normalised_noise + base_elevation + elevation_offset).clamp(0., 1.);

      // Adjust noise based on distance from center using falloff map
      let distance_x = (tx - center.x).abs() as f64 / max_distance;
      let distance_y = (ty - center.y).abs() as f64 / max_distance;
      let distance_from_center = distance_x.max(distance_y);
      let falloff = (1. - distance_from_center).max(0.).powf(falloff_strength);
      let adjusted_noise = normalised_noise * falloff;

      // Create debug data for troubleshooting
      let debug_data = DebugData {
        noise: normalised_noise,
        noise_elevation_offset: elevation_offset,
      };

      // Determine terrain type based on noise
      let tile = match adjusted_noise {
        n if n > 0.75 => DraftTile::new(ig, tg, TerrainType::Forest, debug_data),
        n if n > 0.6 => DraftTile::new(ig, tg, TerrainType::Grass, debug_data),
        n if n > 0.45 => DraftTile::new(ig, tg, TerrainType::Sand, debug_data),
        n if n > 0.3 => DraftTile::new(ig, tg, TerrainType::Shore, debug_data),
        _ => DraftTile::new(ig, tg, TerrainType::Water, debug_data),
      };

      tiles[ix as usize][iy as usize] = Some(tile);
      ix += 1;
    }
    iy += 1;
    ix = 0;
  }
  trace!(
    "Generated draft chunk at {:?} in {} ms on [{}]",
    tg,
    get_time() - start_time,
    async_utils::thread_name()
  );

  tiles
}
