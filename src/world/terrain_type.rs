#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Hash)]
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
