#![allow(dead_code)]

use crate::coords::Point;
use crate::coords::point::{ChunkGrid, TileGrid, World};
use bevy::color::Color;
use bevy::math::UVec2;
use std::ops::Range;

// ------------------------------------------------------------------------------------------------------
// Settings: General
pub const DRAW_GIZMOS: bool = false;
pub const GENERATE_NEIGHBOUR_CHUNKS: bool = true;
pub const ENABLE_TILE_DEBUGGING: bool = true;
pub const DRAW_TERRAIN_SPRITES: bool = true;
pub const ANIMATE_TERRAIN_SPRITES: bool = true;
pub const SPAWN_UP_TO_LAYER: usize = 4;
pub const SPAWN_FROM_LAYER: usize = 0;
pub const ENABLE_WORLD_PRUNING: bool = true;
// ------------------------------------------------------------------------------------------------------
// Settings: Metadata
pub const METADATA_GRID_APOTHEM: i32 = 3;
pub const ELEVATION_CHUNK_STEP_SIZE: f64 = 0.2;
pub const ELEVATION_OFFSET: f64 = 0.6;
pub const BIOME_NOISE_FREQUENCY: f64 = 0.1;
pub const BIOME_IS_ROCKY_PROBABILITY: f64 = 0.3;
// ------------------------------------------------------------------------------------------------------
// Settings: World
pub const NOISE_SEED: u32 = 1;
pub const NOISE_STRENGTH: f64 = 0.75;
pub const NOISE_OCTAVES: usize = 3;
pub const NOISE_FREQUENCY: f64 = 0.07;
pub const NOISE_PERSISTENCE: f64 = 0.7;
pub const NOISE_AMPLITUDE: f64 = 4.5;
pub const FALLOFF_STRENGTH: f64 = 2.5;
pub const FALLOFF_NOISE_STRENGTH: f64 = 0.5;
// ------------------------------------------------------------------------------------------------------
// Settings: Objects
pub const GENERATE_OBJECTS: bool = true;
pub const ENABLE_COLOUR_VARIATIONS: bool = false;
// ------------------------------------------------------------------------------------------------------
// Chunks and tiles
pub const MAX_CHUNKS: usize = 9;
/// The size of a buffer around a chunk that is generated but not rendered. Must be 1, always.
pub const BUFFER_SIZE: i32 = 1;
/// The size of a chunk, including a border that will not be rendered. This is to ensure that the
/// [`crate::generation::lib::TileType`]s of outermost tiles are known. Must not be modified directly. Change
/// [`CHUNK_SIZE`] instead.
pub const CHUNK_SIZE_PLUS_BUFFER: i32 = CHUNK_SIZE + 2 * BUFFER_SIZE;
/// The size of a chunk that is rendered on the screen.
pub const CHUNK_SIZE: i32 = 16;
pub const ORIGIN_CHUNK_GRID_SPAWN_POINT: Point<ChunkGrid> = Point::new_const(0, 0);
pub const ORIGIN_WORLD_SPAWN_POINT: Point<World> =
  Point::new_const(-(CHUNK_SIZE / 2) * TILE_SIZE as i32, (CHUNK_SIZE / 2) * TILE_SIZE as i32);
pub const ORIGIN_TILE_GRID_SPAWN_POINT: Point<TileGrid> = Point::new_const(-(CHUNK_SIZE / 2), CHUNK_SIZE / 2);
pub const DESPAWN_DISTANCE: f32 = CHUNK_SIZE as f32 * TILE_SIZE as f32 * 1.75;
// ------------------------------------------------------------------------------------------------------
// Tiles
pub const TILE_SIZE: u32 = 32;
pub const WATER_LAYER: usize = 0;
pub const SHORE_LAYER: usize = 1;
pub const SAND_LAYER: usize = 2;
pub const GRASS_LAYER: usize = 3;
pub const FOREST_LAYER: usize = 4;
// ------------------------------------------------------------------------------------------------------
// Sprites: Placeholder tile set
pub const TILE_SET_PLACEHOLDER_PATH: &str = "tilesets/default.png";
pub const TILE_SET_PLACEHOLDER_COLUMNS: u32 = 5;
pub const TILE_SET_PLACEHOLDER_ROWS: u32 = 1;
// ------------------------------------------------------------------------------------------------------
// Sprites: Detailed tile sets
pub const TS_WATER_PATH: &str = "tilesets/water-deep.png";
pub const TS_SHORE_PATH: &str = "tilesets/water-shallow.png";
pub const TS_LAND_HUMID_L1_PATH: &str = "tilesets/land-humid-l1.png";
pub const TS_LAND_HUMID_L2_PATH: &str = "tilesets/land-humid-l2.png";
pub const TS_LAND_HUMID_L3_PATH: &str = "tilesets/land-humid-l3.png";
pub const TS_LAND_MODERATE_L1_PATH: &str = "tilesets/land-moderate-l1.png";
pub const TS_LAND_MODERATE_L2_PATH: &str = "tilesets/land-moderate-l2.png";
pub const TS_LAND_MODERATE_L3_PATH: &str = "tilesets/land-moderate-l3.png";
pub const TS_LAND_DRY_L1_PATH: &str = "tilesets/land-dry-l1.png";
pub const TS_LAND_DRY_L2_PATH: &str = "tilesets/land-dry-l2.png";
pub const TS_LAND_DRY_L3_PATH: &str = "tilesets/land-dry-l3.png";
pub const TILE_SET_ROWS: u32 = 17;
pub const DEFAULT_STATIC_TILE_SET_COLUMNS: u32 = 1;
pub const DEFAULT_ANIMATED_TILE_SET_COLUMNS: u32 = 4;
pub const ANIMATION_LENGTH: usize = 4;
pub const DEFAULT_ANIMATION_FRAME_DURATION: f32 = 0.5;
// ------------------------------------------------------------------------------------------------------
// Sprites: Detailed tile set sprite indices
pub const FILL: usize = 4;
pub const INNER_CORNER_BOTTOM_LEFT: usize = 2;
pub const INNER_CORNER_BOTTOM_RIGHT: usize = 0;
pub const INNER_CORNER_TOP_LEFT: usize = 8;
pub const INNER_CORNER_TOP_RIGHT: usize = 6;
pub const OUTER_CORNER_BOTTOM_LEFT: usize = 9;
pub const OUTER_CORNER_BOTTOM_RIGHT: usize = 10;
pub const OUTER_CORNER_TOP_LEFT: usize = 12;
pub const OUTER_CORNER_TOP_RIGHT: usize = 11;
pub const TOP_LEFT_TO_BOTTOM_RIGHT_BRIDGE: usize = 13;
pub const TOP_RIGHT_TO_BOTTOM_LEFT_BRIDGE: usize = 14;
pub const TOP_FILL: usize = 7;
pub const BOTTOM_FILL: usize = 1;
pub const RIGHT_FILL: usize = 3;
pub const LEFT_FILL: usize = 5;
pub const SINGLE: usize = 15;
pub const ERROR: usize = 16;
// ------------------------------------------------------------------------------------------------------
// Objects
pub const TREES_HUMID_OBJ_PATH: &str = "objects/trees-humid.png";
pub const TREES_MODERATE_OBJ_PATH: &str = "objects/trees-moderate.png";
pub const TREES_DRY_OBJ_PATH: &str = "objects/trees-dry.png";
pub const TREES_OBJ_COLUMNS: u32 = 6;
pub const TREES_OBJ_ROWS: u32 = 1;
pub const TREES_OBJ_SIZE: UVec2 = UVec2::new(64, 128);
pub const WATER_DEEP_OBJ_PATH: &str = "objects/objects-water-deep.png";
pub const WATER_SHALLOW_OBJ_PATH: &str = "objects/objects-water-shallow.png";
pub const OBJ_L1_DRY_PATH: &str = "objects/objects-l1-dry.png";
pub const OBJ_L1_MODERATE_PATH: &str = "objects/objects-l1-moderate.png";
pub const OBJ_L1_HUMID_PATH: &str = "objects/objects-l1-humid.png";
pub const OBJ_L2_DRY_PATH: &str = "objects/objects-l2-dry.png";
pub const OBJ_L2_MODERATE_PATH: &str = "objects/objects-l2-moderate.png";
pub const OBJ_L2_HUMID_PATH: &str = "objects/objects-l2-humid.png";
pub const OBJ_L3_DRY_PATH: &str = "objects/objects-l3-dry.png";
pub const OBJ_L3_MODERATE_PATH: &str = "objects/objects-l3-moderate.png";
pub const OBJ_L3_HUMID_PATH: &str = "objects/objects-l3-humid.png";
pub const DEFAULT_OBJ_COLUMNS: u32 = 16;
pub const DEFAULT_OBJ_ROWS: u32 = 2;
pub const DEFAULT_OBJ_SIZE: UVec2 = UVec2::new(32, 32);
// ------------------------------------------------------------------------------------------------------
// Colours
pub const RED: Color = Color::hsl(0.59, 0.32, 0.52);
pub const PURPLE: Color = Color::srgb(0.706, 0.557, 0.678);
pub const YELLOW: Color = Color::srgb(0.922, 0.796, 0.545);
pub const ORANGE: Color = Color::srgb(0.816, 0.529, 0.439);
pub const GREEN: Color = Color::srgb(0.639, 0.745, 0.549);
pub const WATER_BLUE: Color = Color::srgb(0.305882, 0.611765, 0.74902);
pub const DEEP_WATER_BLUE: Color = Color::srgb(0.259, 0.471, 0.565);
pub const LIGHT: Color = Color::srgb(0.925, 0.937, 0.957);
pub const MEDIUM: Color = Color::srgb(0.60, 0.639, 0.714);
pub const DARK: Color = Color::srgb(0.298, 0.337, 0.416);
pub const VERY_DARK: Color = Color::srgb(0.12, 0.14, 0.18);
pub const RGB_COLOUR_VARIATION: f32 = 0.2;
pub const DARKNESS_RANGE: Range<f32> = 0.0..0.2;
pub const BRIGHTNESS_RANGE: Range<f32> = 0.0..0.4;
// ------------------------------------------------------------------------------------------------------
// Window
pub const WINDOW_WIDTH: f32 = 1280.;
pub const WINDOW_HEIGHT: f32 = 720.;
// ------------------------------------------------------------------------------------------------------
// Common errors
pub const TERRAIN_TYPE_ERROR: &'static str = "Invalid terrain type for drawing a terrain sprite";
