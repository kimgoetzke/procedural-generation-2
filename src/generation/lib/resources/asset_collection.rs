use crate::generation::lib::TileType;
use crate::generation::lib::resources::asset_pack::AssetPack;
use bevy::platform::collections::HashSet;

/// An asset collection groups together related asset packs and related "metadata" in a context where animations are
/// used. It is used to pre-initialise and bundle resources that are used repeatedly when spawning sprites.
#[derive(Default, Debug, Clone)]
pub struct AssetCollection {
  pub stat: AssetPack,
  pub anim: Option<AssetPack>,
  pub animated_tile_types: HashSet<TileType>,
  /// The index offset describes the number of sprites by which to shift in a sprite sheet to get to the next sprite of
  /// a different type when the sprite sheet contains animations. Its value is the number of columns in the sprite
  /// sheet and, therefore, the number of frames in the animation.
  ///
  /// Example: Imagine a sprite sheet with two sprites,
  /// a roof tile, followed by a wall tile, each with a 3-frame animation. In this example, the offset would be 3 and
  /// it would allow you to go from the first frame of the roof tile (index 0) to the first frame of the wall tile
  /// (index 3) by adding the offset.
  pub index_offset: usize,
}

impl AssetCollection {
  pub fn index_offset(&self) -> usize {
    self.index_offset
  }
}
