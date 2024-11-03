use crate::constants::{BUFFER_SIZE, CHUNK_SIZE_PLUS_BUFFER};
use crate::coords::point::{TileGrid, World};
use crate::coords::{Coords, Point};
use crate::generation::lib::{DraftTile, TerrainType};
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
  pub fn new(tg: Point<TileGrid>, settings: &Settings) -> Self {
    let data = generate_terrain_data(&tg, settings);
    Self {
      center: Point::new_world(tg.x + (CHUNK_SIZE_PLUS_BUFFER / 2), tg.y + (CHUNK_SIZE_PLUS_BUFFER / 2)),
      coords: Coords::new_for_chunk(tg),
      data,
    }
  }
}

// TODO: Allow for slowly changing terrain (e.g. the further south, the less water there is)
/// Generates terrain data for a draft chunk based on Perlin noise. Expects `tg` to be a `Point` of type
/// `TileGrid` that describes the top-left corner of the grid.
fn generate_terrain_data(tg: &Point<TileGrid>, settings: &Settings) -> Vec<Vec<Option<DraftTile>>> {
  let mut noise_stats: (f64, f64, f64, f64) = (5., -5., 5., -5.);
  let time = get_time();
  let perlin: BasicMulti<Perlin> = BasicMulti::new(settings.world.noise_seed)
    .set_octaves(settings.world.noise_octaves)
    .set_frequency(settings.world.noise_frequency)
    .set_persistence(settings.world.noise_persistence);
  let amplitude = settings.world.noise_amplitude;
  let start = Point::new_tile_grid(tg.x - BUFFER_SIZE, tg.y + BUFFER_SIZE);
  let end = Point::new_tile_grid(start.x + CHUNK_SIZE_PLUS_BUFFER - 1, start.y - CHUNK_SIZE_PLUS_BUFFER + 1);
  let center = Point::new_tile_grid((start.x + end.x) / 2, (start.y + end.y) / 2);
  let max_distance = (CHUNK_SIZE_PLUS_BUFFER as f64) / 2.;
  let elevation = settings.world.elevation;
  let falloff_strength = settings.world.falloff_strength;
  let mut tiles = vec![vec![None; CHUNK_SIZE_PLUS_BUFFER as usize]; CHUNK_SIZE_PLUS_BUFFER as usize];
  let mut ix = 0;
  let mut iy = 0;

  for ty in (end.y..=start.y).rev() {
    for tx in start.x..=end.x {
      let tg = Point::new_tile_grid(tx, ty);
      let ig = Point::new_internal_grid(ix, iy);

      // Calculate noise value
      let noise = perlin.get([tx as f64, ty as f64]);
      let clamped_noise = (noise * amplitude).clamp(-1., 1.);
      let normalised_noise = (clamped_noise + 1.) / 2.;
      let normalised_noise = (normalised_noise + elevation).clamp(0., 1.);

      // Adjust noise based on distance from center using falloff map
      let distance_x = (tx - center.x).abs() as f64 / max_distance;
      let distance_y = (ty - center.y).abs() as f64 / max_distance;
      let distance_from_center = distance_x.max(distance_y);
      let falloff = (1. - distance_from_center).max(0.).powf(falloff_strength);
      let adjusted_noise = normalised_noise * falloff;

      // Determine terrain type based on noise
      let tile = match adjusted_noise {
        n if n > 0.75 => DraftTile::new(ig, tg, TerrainType::Forest),
        n if n > 0.6 => DraftTile::new(ig, tg, TerrainType::Grass),
        n if n > 0.45 => DraftTile::new(ig, tg, TerrainType::Sand),
        n if n > 0.3 => DraftTile::new(ig, tg, TerrainType::Shore),
        _ => DraftTile::new(ig, tg, TerrainType::Water),
      };

      noise_stats.0 = noise_stats.0.min(normalised_noise);
      noise_stats.1 = noise_stats.1.max(normalised_noise);
      noise_stats.2 = noise_stats.2.min(adjusted_noise);
      noise_stats.3 = noise_stats.3.max(adjusted_noise);
      trace!("{:?} => Noise: {}", &tile, adjusted_noise);

      tiles[ix as usize][iy as usize] = Some(tile);
      ix += 1;
    }
    iy += 1;
    ix = 0;
  }
  log(tg, &mut noise_stats, time, &mut tiles);

  tiles
}

fn log(tg: &Point<TileGrid>, noise_stats: &mut (f64, f64, f64, f64), time: u128, tiles: &mut Vec<Vec<Option<DraftTile>>>) {
  let mut str = "|".to_string();
  for y in 0..tiles.len() {
    for x in 0..tiles[y].len() {
      if let Some(tile) = &tiles[x][y] {
        str.push_str(&format!(" {:?}", tile.terrain).chars().take(5).collect::<String>());
        str.push_str(" |");
      } else {
        str.push_str("None |");
      }
    }
    trace!("{}", str);
    str = "|".to_string();
  }
  trace!("Noise ranges from {:.2} to {:.2}", noise_stats.0, noise_stats.1);
  trace!("Adjusted noise ranges from {:.2} to {:.2}", noise_stats.2, noise_stats.3);
  trace!(
    "Generated draft chunk at tg{:?} in {} ms on [{}]",
    tg,
    get_time() - time,
    async_utils::get_thread_info()
  );
}
