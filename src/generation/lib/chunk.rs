use crate::constants::*;
use crate::coords::point::{ChunkGrid, TileGrid, World};
use crate::coords::{Coords, Point};
use crate::generation::lib::debug_data::DebugData;
use crate::generation::lib::{shared, Direction, DraftTile, LayeredPlane, TerrainType};
use crate::generation::resources::{BiomeMetadataSet, Metadata};
use crate::resources::Settings;
use bevy::log::*;
use noise::{BasicMulti, MultiFractal, NoiseFn, Perlin};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

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

// TODO: Consider removing this struct
#[derive(Debug, Clone, PartialEq)]
struct Distances {
  top_left: f64,
  top: f64,
  top_right: f64,
  left: f64,
  center: f64,
  right: f64,
  bottom_left: f64,
  bottom: f64,
  bottom_right: f64,
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
  let mut rng = StdRng::seed_from_u64(shared::calculate_seed(cg.clone(), settings.world.noise_seed));
  let perlin: BasicMulti<Perlin> = BasicMulti::new(settings.world.noise_seed)
    .set_octaves(settings.world.noise_octaves)
    .set_frequency(settings.world.noise_frequency)
    .set_persistence(settings.world.noise_persistence);
  let amplitude = settings.world.noise_amplitude;
  let strength = settings.world.noise_strength;
  let start = Point::new_tile_grid(tg.x - BUFFER_SIZE, tg.y + BUFFER_SIZE);
  let end = Point::new_tile_grid(start.x + CHUNK_SIZE_PLUS_BUFFER - 1, start.y - CHUNK_SIZE_PLUS_BUFFER + 1);
  let center = Point::new_tile_grid((start.x + end.x) / 2, (start.y + end.y) / 2);
  let max_distance = (CHUNK_SIZE_PLUS_BUFFER as f64) / 2.;
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
      let normalised_noise = ((normalised_noise * strength) + elevation_offset).clamp(0., 1.);

      // Calculate distances to chunk edge in all directions
      let distances = calculate_distances(start, end, center, max_distance, tx, ty);

      // Calculate if this tile is a biome edge
      let is_biome_edge = is_tile_at_edge_of_biome(ix, iy, distances.center, &biome_metadata, &mut rng);

      // Create debug data for troubleshooting
      let debug_data = DebugData {
        noise: normalised_noise,
        noise_elevation_offset: elevation_offset,
        is_biome_edge,
      };

      // Determine terrain type based on the above
      let terrain = match normalised_noise {
        n if n > 0.75 => TerrainType::new(TerrainType::Land3, is_biome_edge),
        n if n > 0.6 => TerrainType::new(TerrainType::Land2, is_biome_edge),
        n if n > 0.45 => TerrainType::new(TerrainType::Land1, is_biome_edge),
        n if n > 0.3 => TerrainType::new(TerrainType::ShallowWater, is_biome_edge),
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

fn calculate_distances(
  start: Point<TileGrid>,
  end: Point<TileGrid>,
  center: Point<TileGrid>,
  max_distance: f64,
  tx: i32,
  ty: i32,
) -> Distances {
  let distance_x = (tx - center.x).abs() as f64 / max_distance;
  let distance_y = (ty - center.y).abs() as f64 / max_distance;
  let distance_from_center = distance_x.max(distance_y);
  let distances = Distances {
    top_left: (((tx - start.x).pow(2) + (ty - start.y).pow(2)) as f64).sqrt() / max_distance,
    top: (ty - start.y).abs() as f64 / max_distance,
    top_right: (((end.x - tx).pow(2) + (ty - start.y).pow(2)) as f64).sqrt() / max_distance,
    left: (tx - start.x).abs() as f64 / max_distance,
    center: distance_from_center,
    right: (end.x - tx).abs() as f64 / max_distance,
    bottom_left: (((tx - start.x).pow(2) + (end.y - ty).pow(2)) as f64).sqrt() / max_distance,
    bottom: (end.y - ty).abs() as f64 / max_distance,
    bottom_right: (((end.x - tx).pow(2) + (end.y - ty).pow(2)) as f64).sqrt() / max_distance,
  };
  trace!("tg({}, {}): Distances = {:?}", tx, ty, distances);

  distances
}

const INSIDE: i32 = 1;
const OUTSIDE: i32 = CHUNK_SIZE + 1;
const EXPANDED_INSIDE: i32 = 2;
const EXPANDED_OUTSIDE: i32 = CHUNK_SIZE;

/// Calculates if a tile `TerrainType` should be adjusted by checking if:
/// 1. The tile is "far enough" from the center (otherwise it cannot be an edge)
/// 2. The tile is at any of the edges of the chunk (direction match statement arms using `INSIDE` and/or `OUTSIDE`)
/// 3. The tile is at the randomly determined, expanded edges of the chunk (arms using `EXPANDED_INSIDE`,
///    `EXPANDED_OUTSIDE`) - this introduces some randomness (vs having perfectly straight edges around chunks)
///
/// If all of the above checks are true, the tile is located at the edge of a biome, allowing the tile to be forcibly
/// adjusted to a lower `TerrainType`. Without this, you'd need to have a lot of additional sprites to handle the
/// transitions between each possible biome/terrain type/tile type combination (= 144 extra sprites at the time of
/// writing this code).
#[allow(non_contiguous_range_endpoints)]
fn is_tile_at_edge_of_biome(
  ix: i32,
  iy: i32,
  distance_from_center: f64,
  biome_metadata: &BiomeMetadataSet,
  rng: &mut StdRng,
) -> bool {
  if distance_from_center <= 0.6 {
    return false;
  }

  let is_considered_edge = rng.gen_bool(0.3);
  let direction = match (ix, iy, is_considered_edge) {
    (..INSIDE, ..INSIDE, _) => Direction::TopLeft,
    (OUTSIDE.., ..INSIDE, _) => Direction::TopRight,
    (..INSIDE, OUTSIDE.., _) => Direction::BottomLeft,
    (OUTSIDE.., OUTSIDE.., _) => Direction::BottomRight,
    (_, ..INSIDE, _) => Direction::Top,
    (_, OUTSIDE.., _) => Direction::Bottom,
    (OUTSIDE.., _, _) => Direction::Right,
    (..INSIDE, _, _) => Direction::Left,
    (EXPANDED_INSIDE..EXPANDED_OUTSIDE, ..EXPANDED_INSIDE, true) => Direction::Top,
    (EXPANDED_INSIDE..EXPANDED_OUTSIDE, EXPANDED_OUTSIDE.., true) => Direction::Bottom,
    (EXPANDED_OUTSIDE.., EXPANDED_INSIDE..EXPANDED_OUTSIDE, true) => Direction::Right,
    (..EXPANDED_INSIDE, EXPANDED_INSIDE..EXPANDED_OUTSIDE, true) => Direction::Left,
    _ => Direction::Center,
  };

  direction != Direction::Center && !biome_metadata.is_same_climate(&direction)
}
