use crate::generation::object::lib::ObjectName;
use strum::EnumCount;

const OBJECT_NAME_SET_WORDS: usize = ObjectName::COUNT.div_ceil(u64::BITS as usize);

/// Just an array of numbers representing [`ObjectName`]s. Used by the wave function collapse algorithm when
/// determining the permitted states of a cell. This struct only exists as a performance optimisation. This bitset
/// implementation uses fixed memory, performs no hashing (unlike `HashSet<ObjectName>`), and needs no heap allocation.
#[derive(Clone, Copy, Debug, Default)]
pub struct PermittedObjectNames([u64; OBJECT_NAME_SET_WORDS]);

impl PermittedObjectNames {
  pub fn insert(&mut self, name: ObjectName) {
    let index = name as usize;
    let word = index / u64::BITS as usize;
    let bit = index % u64::BITS as usize;
    self.0[word] |= 1 << bit;
  }

  pub fn contains(&self, name: ObjectName) -> bool {
    let index = name as usize;
    let word = index / u64::BITS as usize;
    let bit = index % u64::BITS as usize;
    self.0[word] & (1 << bit) != 0
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use strum::IntoEnumIterator;

  #[test]
  fn insert_keeps_every_object_and_settlement_tile_distinct() {
    let names: Vec<_> = ObjectName::iter().collect();
    for &inserted in &names {
      let mut permitted = PermittedObjectNames::default();
      assert!(!permitted.contains(inserted));
      permitted.insert(inserted);
      permitted.insert(inserted);
      for &candidate in &names {
        assert_eq!(
          permitted.contains(candidate),
          inserted == candidate,
          "Inserted {inserted:?}, checked {candidate:?}"
        );
      }
    }
  }

  #[test]
  fn insert_preserves_names_across_word_boundaries() {
    let mut permitted = PermittedObjectNames::default();
    let names: Vec<_> = ObjectName::iter().collect();
    for &name in &names {
      permitted.insert(name);
    }
    for name in names {
      assert!(permitted.contains(name), "Missing {name:?}");
    }
  }
}
