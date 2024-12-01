use crate::constants::*;
use crate::coords::point::{ChunkGrid, TileGrid, World};
use crate::coords::{Coords, Point};
use crate::generation::lib::debug_data::DebugData;
use crate::generation::lib::{shared, Direction, DraftTile, LayeredPlane, TerrainType};
use crate::generation::resources::{BiomeMetadataSet, Metadata};
use crate::resources::Settings;
use bevy::log::*;
use noise::{BasicMulti, MultiFractal, NoiseFn, Perlin};

/// A `Chunk` represents a single chunk of the world.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Chunk {
  pub coords: Coords,
  pub center: Point<World>,
  pub layered_plane: LayeredPlane,
}

impl Chunk {
  /// Creates a new chunk from a draft chunk by converting the flat terrain data from the draft chunk into a
  /// `LayeredPlane`. As a result, a chunk has multiple layers of terrain data, each of which contains rich information
  /// about the `Tile`s that make up the terrain including their `TileType`s.
  pub fn new(w: Point<World>, tg: Point<TileGrid>, metadata: &Metadata, settings: &Settings) -> Self {
    let coords = Coords::new_for_chunk(w, tg);
    let data = generate_terrain_data(&tg, &coords.chunk_grid, metadata, settings);
    let layered_plane = LayeredPlane::new(data, settings);
    Chunk {
      coords,
      center: Point::new_world(tg.x + (CHUNK_SIZE_PLUS_BUFFER / 2), tg.y + (CHUNK_SIZE_PLUS_BUFFER / 2)),
      layered_plane,
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
  let biome_metadata = metadata.get_biome_metadata_for(cg);
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

      // Calculate distance from center for falloff maps
      let distance_from_center = calculate_distance_from_center(center, max_distance, tx, ty);

      // TODO: Refactor to only use falloff value if the neighbour has a different max layer cap
      // Calculate terrain falloff map value, if max layer cap is enabled
      let terrain_falloff = calculate_terrain_falloff(
        use_max_layer_cap,
        falloff_strength,
        falloff_noise_strength,
        clamped_noise,
        distance_from_center,
      );

      let is_biome_edge = calculate_biome_falloff(ix, iy, distance_from_center, &biome_metadata, cg);

      // Create debug data for troubleshooting
      let debug_data = DebugData {
        noise: normalised_noise,
        noise_elevation_offset: elevation_offset,
        is_biome_edge,
      };
      // Determine terrain type based on noise
      let max_layer = biome_metadata.this.max_layer;
      let terrain = match normalised_noise {
        n if n > 0.75 => TerrainType::new(TerrainType::Land3, max_layer, terrain_falloff, is_biome_edge),
        n if n > 0.6 => TerrainType::new(TerrainType::Land2, max_layer, terrain_falloff, is_biome_edge),
        n if n > 0.45 => TerrainType::new(TerrainType::Land1, max_layer, terrain_falloff, is_biome_edge),
        n if n > 0.3 => TerrainType::new(TerrainType::ShallowWater, max_layer, terrain_falloff, is_biome_edge),
        _ => TerrainType::DeepWater,
      };
      let climate = biome_metadata.this.climate;

      let tile = DraftTile::new(ig, tg, terrain, climate, debug_data);
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

fn calculate_distance_from_center(center: Point<TileGrid>, max_distance: f64, tx: i32, ty: i32) -> f64 {
  let distance_x = (tx - center.x).abs() as f64 / max_distance;
  let distance_y = (ty - center.y).abs() as f64 / max_distance;
  let distance_from_center = distance_x.max(distance_y);
  // info!("tg({}, {}): Distance from center = {}", tx, ty, distance_from_center);

  distance_from_center
}

fn calculate_terrain_falloff(
  use_max_layer_cap: bool,
  falloff_strength: f64,
  falloff_noise_strength: f64,
  clamped_noise: f64,
  distance_from_center: f64,
) -> f64 {
  if !use_max_layer_cap {
    return 0.;
  }
  let falloff_noise = clamped_noise * falloff_noise_strength;
  let falloff = (1. - distance_from_center + falloff_noise).max(0.).powf(falloff_strength);

  falloff
}

fn calculate_biome_falloff(
  ix: i32,
  iy: i32,
  distance_from_center: f64,
  biome_metadata: &BiomeMetadataSet,
  cg: &Point<ChunkGrid>,
) -> bool {
  if distance_from_center > 0.5 {
    let direction = match (ix, iy) {
      (..2, ..2) => Direction::TopLeft,
      (CHUNK_SIZE.., ..2) => Direction::TopRight,
      (..2, CHUNK_SIZE..) => Direction::BottomLeft,
      (CHUNK_SIZE.., CHUNK_SIZE..) => Direction::BottomRight,
      (_, ..2) => Direction::Top,
      (_, CHUNK_SIZE..) => Direction::Bottom,
      (CHUNK_SIZE.., _) => Direction::Right,
      (..2, _) => Direction::Left,
      _ => Direction::Center,
    };
    if direction == Direction::Center {
      return false;
    }
    if !biome_metadata.is_same_climate(&direction) {
      info!(
        "Adjusting tile ig({}, {}) because [{:?}] of {} is a different biome",
        ix, iy, direction, cg
      );
      return true;
    }
  }

  false
}
