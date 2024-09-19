use crate::world::tile::DraftTile;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Plane {
  pub(crate) data: Vec<Vec<Option<DraftTile>>>,
}

impl Plane {
  pub fn new(plane: Vec<Vec<Option<DraftTile>>>) -> Self {
    Self { data: plane }
  }

  pub fn empty(size: usize) -> Self {
    let mut plane = Vec::new();
    for _ in 0..size {
      let mut row = Vec::new();
      for _ in 0..size {
        row.push(None);
      }
      plane.push(row);
    }
    Self { data: plane }
  }

  pub fn get(&self, x: i32, y: i32) -> Option<&DraftTile> {
    let i = x as usize;
    let j = y as usize;
    if i < self.data.len() && j < self.data[0].len() {
      self.data[i][j].as_ref()
    } else {
      None
    }
  }
}
