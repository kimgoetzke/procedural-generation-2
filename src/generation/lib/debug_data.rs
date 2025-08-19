use bevy::prelude::Reflect;
use std::fmt;
use std::hash::{Hash, Hasher};

/// This struct is used to store generation data on [`crate::generation::lib::DraftTile`]s which are then converted to
/// [`crate::generation::lib::Tile`]s. The idea is that, with this struct, we can still access stats from the terrain
/// generation process after it is done and visualise it in the UI or log it to the console.
#[derive(Copy, Clone, Reflect)]
pub struct DebugData {
  pub noise: f64,
  pub noise_elevation_offset: f64,
  pub is_biome_edge: bool,
}

impl PartialEq for DebugData {
  fn eq(&self, other: &Self) -> bool {
    self.noise == other.noise && self.noise_elevation_offset == other.noise_elevation_offset
  }
}

impl Eq for DebugData {}

impl Hash for DebugData {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.noise.to_bits().hash(state);
    self.noise_elevation_offset.to_bits().hash(state);
  }
}

impl fmt::Debug for DebugData {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "Debug data printed below:\n\
    ┌────────────────────────┬──────────────┐\n\
    │ Noise                  │ {:12.5} │\n\
    ├────────────────────────┼──────────────┤\n\
    │ Noise elevation offset │ {:12.5} │\n\
    ├────────────────────────┼──────────────┤\n\
    │ Is at edge of biome    │ {:12.5} │\n\
    └────────────────────────┴──────────────┘",
      self.noise, self.noise_elevation_offset, self.is_biome_edge
    )
  }
}
