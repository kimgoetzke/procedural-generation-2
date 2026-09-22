# How to add more assets

## Decorative object sprite assets

1. Add the sprite to the relevant sprite sheet in `assets/objects/`
2. Add a new option to the `ObjectName` enum
3. Optional: Add the object name to the `any.terrain.ruleset.toml` file (top, right, bottom, left) if it can be placed
   next to
   a tile that contains no object (i.e. `ObjectName::Empty`)
4. Add the object name to the `all.tile-type.ruleset.toml` file (like just `Fill`) to the relevant `TileType`s on which
   the object can be placed
5. Add a new state to the relevant `{terrain}.terrain.ruleset.toml` file using the index from the sprite sheet
    - Make sure provide of permitted neighbours (even if just `Empty` on all sides)
    - Make sure the permitted neighbours themselves list the new object name as a neighbour, too
    - The application will run some validations and prevent startup with clear error messages if the configured state is
      unresolvable
6. Optional: Add the object name to any terrain-climate combination in `all.exclusions.ruleset.toml` if it shouldn't
   be placed in those terrains and/or climates
7. Optional: If this is a large asset, make sure to add it to `ObjectName.is_multi_tile()`
8. Optional: If this is an animated asset, add it to `ObjectName.is_animated()`

##  Building or path sprite assets

1. Add the sprite to the relevant sprite sheet in `assets/objects/`
2. Update the column and row values in `constants.rs` for buildings/paths, if necessary
3. Add the new option(s) to the `ObjectName` enum
4. Add the object name(s) to the `is_building()` or `is_path()` function in `object_name.rs`
5. Add the object name(s) to the `get_index()` function in `object_name.rs`
6. Add the object name(s) to the `any.terrain.ruleset.toml` file where appropriate (top, right, bottom, left)
7. If building sprite: Add the object name(s) to relevant `BuildingType` in the `BuildingComponentRegistry`

You can but don't need to update any other ruleset files as buildings and paths are placed prior to decorative objects
and therefore don't need to be considered in the wave function collapse algorithm which uses these rulesets. However,
the addition to the "any ruleset" file results in the neighbouring tile of the new sprite to be empty
(`ObjectName::Empty`). Without this, you'll see errors in the wave function collapse algorithm.