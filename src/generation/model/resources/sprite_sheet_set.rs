use crate::generation::model::TileType;
use crate::generation::model::resources::sprite_sheet::SpriteSheet;
use bevy::platform::collections::HashSet;

/// Groups static and animated [`SpriteSheet`]s with their animation metadata.
#[derive(Default, Debug, Clone)]
pub struct SpriteSheetSet {
  pub static_sheet: SpriteSheet,
  pub animated_sheet: Option<SpriteSheet>,
  pub animated_tile_types: HashSet<TileType>,
  /// The index offset describes the number of sprites by which to shift in a sprite sheet to get to the next sprite of
  /// a different type when the sprite sheet contains animations. Its value is the number of columns in the sprite
  /// sheet and, therefore, the number of frames in the animation.
  ///
  /// Example: Imagine a sprite sheet with two sprites, a roof tile, followed by a wall tile, each with a 3-frame
  /// animation. In this example, the offset would be 3, and it would allow you to go from the first frame of the roof
  /// tile (index 0) to the first frame of the wall tile (index 3) by adding the offset.
  pub index_offset: usize,
}

impl SpriteSheetSet {
  /// Returns the offset between sprite types in the sheet.
  pub const fn index_offset(&self) -> usize {
    self.index_offset
  }
}
