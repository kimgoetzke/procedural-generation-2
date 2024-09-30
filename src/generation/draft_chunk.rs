use crate::constants::{BUFFER_SIZE, CHUNK_SIZE_PLUS_BUFFER};
use crate::coords::{Coords, Point};
use crate::generation::get_time;
use crate::generation::terrain_type::TerrainType;
use crate::generation::tile::DraftTile;
use crate::resources::Settings;
use bevy::log::*;
use bevy::prelude::Res;
use noise::{BasicMulti, MultiFractal, NoiseFn, Perlin};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct DraftChunk {
  pub coords: Coords,
  pub center: Point,
  pub data: Vec<Vec<Option<DraftTile>>>,
}

impl DraftChunk {
  /// Creates a new, flat draft chunk with terrain data based on noise by using Perlin noise.
  pub fn new(world_grid: Point, settings: &Res<Settings>) -> Self {
    let data = generate_terrain_data(&world_grid, settings);
    Self {
      center: Point::new_world(
        world_grid.x + (CHUNK_SIZE_PLUS_BUFFER / 2),
        world_grid.y + (CHUNK_SIZE_PLUS_BUFFER / 2),
      ),
      coords: Coords::new_for_chunk(world_grid),
      data,
    }
  }
}

/// Generates terrain data for a draft chunk based on Perlin noise. Expects `world_grid` to be a `Point` of type
/// `WorldGrid` that describes the top-left corner of the grid.
fn generate_terrain_data(world_grid: &Point, settings: &Res<Settings>) -> Vec<Vec<Option<DraftTile>>> {
  let mut noise_stats: (f64, f64, f64, f64) = (5., -5., 5., -5.);
  let time = get_time();
  let perlin: BasicMulti<Perlin> = BasicMulti::new(settings.world.noise_seed)
    .set_octaves(settings.world.noise_octaves)
    .set_frequency(settings.world.noise_frequency)
    .set_persistence(settings.world.noise_persistence);
  let amplitude = settings.world.noise_amplitude;
  let start = Point::new_world_grid(world_grid.x - BUFFER_SIZE, world_grid.y + BUFFER_SIZE);
  let end = Point::new_world_grid(start.x + CHUNK_SIZE_PLUS_BUFFER - 1, start.y - CHUNK_SIZE_PLUS_BUFFER + 1);
  let center = Point::new_world_grid((start.x + end.x) / 2, (start.y + end.y) / 2);
  let max_distance = (CHUNK_SIZE_PLUS_BUFFER as f64) / 2.;
  let elevation = settings.world.elevation;
  let falloff_strength = settings.world.falloff_strength;
  let mut tiles = vec![vec![None; CHUNK_SIZE_PLUS_BUFFER as usize]; CHUNK_SIZE_PLUS_BUFFER as usize];
  let mut cx = 0;
  let mut cy = 0;

  for gy in (end.y..=start.y).rev() {
    for gx in start.x..=end.x {
      let world_grid = Point::new_world_grid(gx, gy);
      let chunk_grid = Point::new_chunk_grid(cx, cy);

      // Calculate noise value
      let noise = perlin.get([gx as f64, gy as f64]);
      let clamped_noise = (noise * amplitude).clamp(-1., 1.);
      let normalised_noise = (clamped_noise + 1.) / 2.;
      let normalised_noise = (normalised_noise + elevation).clamp(0., 1.);

      // Adjust noise based on distance from center using falloff map
      let distance_x = (gx - center.x).abs() as f64 / max_distance;
      let distance_y = (gy - center.y).abs() as f64 / max_distance;
      let distance_from_center = distance_x.max(distance_y);
      let falloff = (1. - distance_from_center).max(0.).powf(falloff_strength);
      let adjusted_noise = normalised_noise * falloff;

      // Determine terrain type based on noise
      let tile = match adjusted_noise {
        n if n > 0.75 => DraftTile::new(chunk_grid, world_grid, TerrainType::Forest),
        n if n > 0.6 => DraftTile::new(chunk_grid, world_grid, TerrainType::Grass),
        n if n > 0.45 => DraftTile::new(chunk_grid, world_grid, TerrainType::Sand),
        n if n > 0.3 => DraftTile::new(chunk_grid, world_grid, TerrainType::Shore),
        _ => DraftTile::new(chunk_grid, world_grid, TerrainType::Water),
      };

      noise_stats.0 = noise_stats.0.min(normalised_noise);
      noise_stats.1 = noise_stats.1.max(normalised_noise);
      noise_stats.2 = noise_stats.2.min(adjusted_noise);
      noise_stats.3 = noise_stats.3.max(adjusted_noise);
      trace!("{:?} => Noise: {}", &tile, adjusted_noise);

      tiles[cx as usize][cy as usize] = Some(tile);
      cx += 1;
    }
    cy += 1;
    cx = 0;
  }
  log(world_grid, &mut noise_stats, time, &mut tiles);

  tiles
}

fn log(world_grid: &Point, noise_stats: &mut (f64, f64, f64, f64), time: u128, tiles: &mut Vec<Vec<Option<DraftTile>>>) {
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
  debug!("Generated draft chunk at {:?} within {} ms", world_grid, get_time() - time);
}
