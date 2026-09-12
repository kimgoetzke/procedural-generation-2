use crate::constants::CHUNK_SIZE;
use crate::coords::Point;
use crate::coords::point::InternalGrid;
use crate::generation::object::lib::Cell;
use rand::RngExt;
use rand::prelude::StdRng;

/// Indexes uncollapsed cells by entropy for constant-time observation.
#[derive(Debug)]
pub(super) struct CellEntropyIndex {
  buckets: Vec<Vec<Point<InternalGrid>>>,
  memberships: Vec<Option<Membership>>,
  lowest: usize,
}

#[derive(Clone, Copy, Debug)]
struct Membership {
  entropy: usize,
  position: usize,
}

impl CellEntropyIndex {
  /// Builds an index from the current grid state.
  pub(super) fn from_cells<'cell>(cells: impl Iterator<Item = &'cell Cell>) -> Self {
    let cells = cells.filter(|cell| !cell.is_collapsed()).collect::<Vec<_>>();
    let maximum_entropy = cells.iter().map(|cell| cell.get_entropy()).max().unwrap_or_default();
    let mut index = Self {
      buckets: vec![Vec::new(); maximum_entropy.saturating_add(1)],
      memberships: vec![None; (CHUNK_SIZE * CHUNK_SIZE) as usize],
      lowest: usize::MAX,
    };
    for cell in cells {
      index.insert(cell.ig, cell.get_entropy());
    }

    index
  }

  /// Chooses a random cell from the lowest non-empty entropy bucket.
  pub(super) fn choose_lowest_entropy_cell(&self, rng: &mut StdRng) -> Option<Point<InternalGrid>> {
    let bucket = self.buckets.get(self.lowest)?;
    let index = rng.random_range(0..bucket.len());
    bucket.get(index).copied()
  }

  /// Updates membership after a cell changes.
  pub(super) fn update(&mut self, cell: &Cell) {
    self.remove(cell.ig);
    if !cell.is_collapsed() {
      self.insert(cell.ig, cell.get_entropy());
    }
  }

  /// Rebuilds membership after restoring a grid snapshot.
  pub(super) fn rebuild<'cell>(&mut self, cells: impl Iterator<Item = &'cell Cell>) {
    *self = Self::from_cells(cells);
  }

  fn insert(&mut self, ig: Point<InternalGrid>, entropy: usize) {
    debug_assert!(entropy > 0, "uncollapsed cells must have positive entropy");
    if entropy >= self.buckets.len() {
      self.buckets.resize_with(entropy + 1, Vec::new);
    }
    let position = self.buckets[entropy].len();
    self.buckets[entropy].push(ig);
    self.memberships[cell_index(ig)] = Some(Membership { entropy, position });
    self.lowest = self.lowest.min(entropy);
  }

  fn remove(&mut self, ig: Point<InternalGrid>) {
    let Some(membership) = self.memberships[cell_index(ig)].take() else {
      return;
    };
    let bucket = &mut self.buckets[membership.entropy];
    bucket.swap_remove(membership.position);
    if let Some(moved_ig) = bucket.get(membership.position).copied()
      && let Some(moved_membership) = &mut self.memberships[cell_index(moved_ig)]
    {
      moved_membership.position = membership.position;
    }
    if membership.entropy == self.lowest && bucket.is_empty() {
      self.lowest = self
        .buckets
        .iter()
        .enumerate()
        .skip(1)
        .find_map(|(entropy, bucket)| (!bucket.is_empty()).then_some(entropy))
        .unwrap_or(usize::MAX);
    }
  }
}

fn cell_index(ig: Point<InternalGrid>) -> usize {
  debug_assert!(ig.x >= 0 && ig.x < CHUNK_SIZE && ig.y >= 0 && ig.y < CHUNK_SIZE);
  (ig.y * CHUNK_SIZE + ig.x) as usize
}

#[cfg(test)]
mod tests {
  use super::CellEntropyIndex;
  use crate::generation::lib::{TerrainType, TileType};
  use crate::generation::object::lib::{Cell, ObjectName, TerrainState};
  use rand::SeedableRng;
  use rand::prelude::StdRng;

  #[test]
  fn chooses_a_cell_from_the_lowest_non_empty_entropy_bucket() {
    let cells = [cell_with_entropy(0, 3), cell_with_entropy(1, 1), cell_with_entropy(2, 2)];
    let entropy_index = CellEntropyIndex::from_cells(cells.iter());
    let mut rng = StdRng::seed_from_u64(1);

    assert_eq!(entropy_index.choose_lowest_entropy_cell(&mut rng), Some(cells[1].ig));
  }

  #[test]
  fn moves_a_cell_when_its_entropy_changes() {
    let cells = [cell_with_entropy(0, 1), cell_with_entropy(1, 2)];
    let mut entropy_index = CellEntropyIndex::from_cells(cells.iter());
    entropy_index.update(&cell_with_entropy(0, 3));
    let mut rng = StdRng::seed_from_u64(1);

    assert_eq!(entropy_index.choose_lowest_entropy_cell(&mut rng), Some(cells[1].ig));
  }

  #[test]
  fn updates_the_membership_of_a_cell_moved_by_swap_remove() {
    let cells = [cell_with_entropy(0, 1), cell_with_entropy(1, 1), cell_with_entropy(2, 2)];
    let mut entropy_index = CellEntropyIndex::from_cells(cells.iter());
    entropy_index.update(&cell_with_entropy(0, 3));
    entropy_index.update(&cell_with_entropy(1, 4));
    let mut rng = StdRng::seed_from_u64(1);

    assert_eq!(entropy_index.choose_lowest_entropy_cell(&mut rng), Some(cells[2].ig));
  }

  #[test]
  fn removes_a_cell_when_it_collapses() {
    let mut cell = cell_with_entropy(0, 1);
    let mut entropy_index = CellEntropyIndex::from_cells(std::iter::once(&cell));
    let mut rng = StdRng::seed_from_u64(1);
    cell.collapse(&mut rng);
    entropy_index.update(&cell);

    assert_eq!(entropy_index.choose_lowest_entropy_cell(&mut rng), None);
  }

  #[test]
  fn rebuild_replaces_all_tracked_cells() {
    let initial_cell = cell_with_entropy(0, 1);
    let replacement_cell = cell_with_entropy(1, 2);
    let mut entropy_index = CellEntropyIndex::from_cells(std::iter::once(&initial_cell));
    entropy_index.rebuild(std::iter::once(&replacement_cell));
    let mut rng = StdRng::seed_from_u64(1);

    assert_eq!(entropy_index.choose_lowest_entropy_cell(&mut rng), Some(replacement_cell.ig));
  }

  fn cell_with_entropy(x: i32, entropy: usize) -> Cell {
    let states = (0..entropy)
      .map(|_| TerrainState::new_with_no_neighbours(ObjectName::Empty, 0, 1))
      .collect::<Vec<_>>();
    let mut cell = Cell::new(x, 0);
    cell.initialise(TerrainType::Any, TileType::Fill, &states, vec![], false);
    cell
  }
}
