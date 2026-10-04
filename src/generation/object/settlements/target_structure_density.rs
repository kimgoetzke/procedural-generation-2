use crate::coordinates::Point;
use crate::coordinates::point::ChunkGrid;
use crate::generation::model::Metadata;

/// Describes how many structures should the algorithms aim to generate, attempting to achieve the density target.
#[derive(Debug)]
pub enum TargetedStructureDensity {
  VeryLow,
  Low,
  Medium,
  High,
  VeryHigh,
}

impl TargetedStructureDensity {
  /// Determines the target density for a settlement from the number of settled neighbours. The more neighbours are
  /// marked as settled, the higher the density of structures should be on this chunk.
  pub fn from(cg: &Point<ChunkGrid>, metadata: &Metadata) -> TargetedStructureDensity {
    let statuses = metadata.get_neighbour_settlement_statuses(cg);
    match statuses.iter().filter(|(_, is_settled)| *is_settled).count() {
      0 => TargetedStructureDensity::VeryLow,
      1 => TargetedStructureDensity::Low,
      2 => TargetedStructureDensity::Medium,
      3 => TargetedStructureDensity::High,
      4.. => TargetedStructureDensity::VeryHigh,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn from_maps_settled_cardinal_neighbour_count_to_size() {
    let cg = Point::new_chunk_grid(3, -2);
    let neighbours = [
      Point::new_chunk_grid(3, -1),
      Point::new_chunk_grid(2, -2),
      Point::new_chunk_grid(4, -2),
      Point::new_chunk_grid(3, -3),
    ];

    for settled_count in 0..=4 {
      let mut metadata = Metadata::default(cg);
      // The chunk itself and a diagonal must not affect its size.
      metadata.settlement.insert(cg, true);
      metadata.settlement.insert(Point::new_chunk_grid(4, -1), true);
      for (index, neighbour) in neighbours.iter().enumerate() {
        metadata.settlement.insert(*neighbour, index < settled_count);
      }

      let size = TargetedStructureDensity::from(&cg, &metadata);

      assert!(
        matches!(
          (settled_count, size),
          (0, TargetedStructureDensity::VeryLow)
            | (1, TargetedStructureDensity::Low)
            | (2, TargetedStructureDensity::Medium)
            | (3, TargetedStructureDensity::High)
            | (4, TargetedStructureDensity::VeryHigh)
        ),
        "Incorrect settlement size for {settled_count} settled cardinal neighbours"
      );
    }
  }
}
