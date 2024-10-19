use crate::constants::CHUNK_SIZE;
use crate::coords::point::ChunkGrid;
use crate::coords::Point;
use crate::generation::lib::TerrainType;
use crate::generation::object::lib::connection_type::get_connection_points;
use crate::generation::object::lib::{Cell, Connection};
use crate::generation::resources::RuleSet;
use bevy::log::*;

#[derive(Debug, Clone)]
pub struct ObjectGrid {
  pub terrain: TerrainType,
  pub grid: Vec<Vec<Cell>>,
}

impl ObjectGrid {
  pub fn new(rule_set: &RuleSet) -> Self {
    let grid = (0..CHUNK_SIZE)
      .map(|y| (0..CHUNK_SIZE).map(|x| Cell::new(x, y, rule_set)).collect())
      .collect();
    ObjectGrid {
      terrain: rule_set.terrain,
      grid,
    }
  }

  pub fn get_neighbours(&mut self, point: &Point<ChunkGrid>) -> Vec<(Connection, &Cell)> {
    let points: Vec<_> = get_connection_points(point).into_iter().collect();
    let mut neighbours = vec![];
    for (direction, point) in points {
      if let Some(cell) = self.grid.iter().flatten().filter(|cell| cell.cg == point).next() {
        neighbours.push((direction, cell));
      }
    }
    trace!("Found {} neighbours for cg{:?}", neighbours.len(), point);

    neighbours
  }

  pub fn get_cell_mut(&mut self, point: &Point<ChunkGrid>) -> Option<&mut Cell> {
    self.grid.iter_mut().flatten().find(|cell| cell.cg == *point)
  }

  /// Replaces the `Cell` at the given point with the provided `Cell`.
  pub fn set_cell(&mut self, cell: Cell) {
    if let Some(existing_cell) = self.grid.iter_mut().flatten().find(|c| c.cg == cell.cg) {
      *existing_cell = cell;
    } else {
      error!("Failed to find cell to update at cg{:?}", cell.cg);
    }
  }

  // TODO: Investigate why entropy can increase between runs (with no snapshot being restored)
  pub fn get_cells_with_lowest_entropy(&self) -> Vec<&Cell> {
    let mut lowest_entropy = usize::MAX;
    let mut lowest_entropy_cells = vec![];
    for cell in self.grid.iter().flatten() {
      if !cell.is_collapsed {
        let entropy = cell.entropy;
        if entropy < lowest_entropy {
          lowest_entropy = entropy;
          lowest_entropy_cells = vec![cell];
        } else if entropy == lowest_entropy {
          lowest_entropy_cells.push(cell);
        }
      }
    }
    debug!(
      "Found {} cell(s) with lowest entropy of {}",
      lowest_entropy_cells.len(),
      lowest_entropy
    );

    lowest_entropy_cells
  }

  pub fn restore_from_snapshot(&mut self, other: &ObjectGrid) {
    self.grid = other.grid.clone();
  }
}
