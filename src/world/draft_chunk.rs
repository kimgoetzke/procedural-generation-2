use crate::constants::CHUNK_SIZE;
use crate::coords::{Coords, Point};
use crate::resources::Settings;
use crate::world::get_time;
use crate::world::terrain_type::TerrainType;
use crate::world::tile::DraftTile;
use bevy::log::trace;
use bevy::prelude::Res;
use noise::{NoiseFn, Perlin};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct DraftChunk {
  pub coords: Coords,
  pub center: Point,
  pub plane: Vec<Vec<Option<DraftTile>>>,
}

impl DraftChunk {
  pub fn new(world_location: Point, settings: &Res<Settings>) -> Self {
    let plane = generate_plane(&world_location, settings);
    Self {
      center: Point::new(world_location.x + (CHUNK_SIZE / 2), world_location.y + (CHUNK_SIZE / 2)),
      coords: Coords::new_for_chunk(world_location),
      plane,
    }
  }
}

fn generate_plane(start: &Point, settings: &Res<Settings>) -> Vec<Vec<Option<DraftTile>>> {
  let mut noise_stats: (f64, f64, f64, f64) = (5., -5., 5., -5.);
  let time = get_time();
  let perlin = Perlin::new(settings.world.noise_seed);
  let end = Point::new(start.x + CHUNK_SIZE - 1, start.y + CHUNK_SIZE - 1);
  let center = Point::new((start.x + end.x) / 2, (start.y + end.y) / 2);
  let max_distance = (CHUNK_SIZE as f64) / 2.;
  let frequency = settings.world.noise_frequency;
  let amplitude = settings.world.noise_amplitude;
  let elevation = settings.world.elevation;
  let falloff_strength = settings.world.falloff_strength;
  let mut tiles = vec![vec![None; CHUNK_SIZE as usize]; CHUNK_SIZE as usize];
  let mut cx = 0;
  let mut cy = 0;

  for gx in start.x..=end.x {
    for gy in start.y..=end.y {
      let chunk_grid = Point::new(cx, cy);
      let tile_grid = Point::new(gx, gy);

      // Calculate noise value
      let noise = perlin.get([gx as f64 * frequency, gy as f64 * frequency]);
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
        n if n > 0.75 => DraftTile::new(chunk_grid, tile_grid, TerrainType::Forest),
        n if n > 0.6 => DraftTile::new(chunk_grid, tile_grid, TerrainType::Grass),
        n if n > 0.45 => DraftTile::new(chunk_grid, tile_grid, TerrainType::Sand),
        n if n > 0.3 => DraftTile::new(chunk_grid, tile_grid, TerrainType::Shore),
        _ => DraftTile::new(chunk_grid, tile_grid, TerrainType::Water),
      };

      noise_stats.0 = noise_stats.0.min(normalised_noise);
      noise_stats.1 = noise_stats.1.max(normalised_noise);
      noise_stats.2 = noise_stats.2.min(adjusted_noise);
      noise_stats.3 = noise_stats.3.max(adjusted_noise);
      trace!("{:?} => Noise: {}", &tile, adjusted_noise);

      tiles[cx as usize][cy as usize] = Some(tile);
      cy += 1;
    }
    cx += 1;
    cy = 0;
  }
  trace!("Noise ranges from {:.2} to {:.2}", noise_stats.0, noise_stats.1);
  trace!("Adjusted noise ranges from {:.2} to {:.2}", noise_stats.2, noise_stats.3);
  trace!("Generated draft chunk at {:?} within {} ms", start, get_time() - time);

  tiles
}
