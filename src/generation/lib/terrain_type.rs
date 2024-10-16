#[derive(serde::Deserialize, Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Hash)]
pub enum TerrainType {
  Water,
  Shore,
  Sand,
  Grass,
  Forest,
  Any,
}

impl Default for TerrainType {
  fn default() -> Self {
    TerrainType::Any
  }
}

impl TerrainType {
  pub fn length() -> usize {
    5 // Ignore TerrainType:Any
  }

  pub fn from(i: usize) -> Self {
    match i {
      0 => TerrainType::Water,
      1 => TerrainType::Shore,
      2 => TerrainType::Sand,
      3 => TerrainType::Grass,
      4 => TerrainType::Forest,
      _ => TerrainType::Any,
    }
  }
}
