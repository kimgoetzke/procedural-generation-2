#![allow(dead_code)]

use crate::coords::point::WorldGrid;
use crate::coords::Point;
use bevy::color::Color;
use bevy::math::UVec2;

// ------------------------------------------------------------------------------------------------------
// Settings: General
pub const DRAW_GIZMOS: bool = false;
pub const GENERATE_NEIGHBOUR_CHUNKS: bool = true;
pub const ENABLE_TILE_DEBUGGING: bool = true;
pub const DRAW_TERRAIN_SPRITES: bool = true;
pub const SPAWN_UP_TO_LAYER: usize = 5;
pub const GENERATE_OBJECTS: bool = true;
// ------------------------------------------------------------------------------------------------------
// Settings: World
pub const NOISE_SEED: u32 = 1;
pub const NOISE_OCTAVES: usize = 4;
pub const NOISE_FREQUENCY: f64 = 0.07;
pub const NOISE_PERSISTENCE: f64 = 0.7;
pub const NOISE_AMPLITUDE: f64 = 8.5;
pub const NOISE_ELEVATION: f64 = -0.05;
pub const FALLOFF_STRENGTH: f64 = 0.;
// ------------------------------------------------------------------------------------------------------
// Settings: Objects
pub const TREE_DENSITY: f64 = 0.5;
// ------------------------------------------------------------------------------------------------------
// Chunks and tiles
/// The size of a buffer around a chunk that is not rendered. Must be 1, always.
pub const BUFFER_SIZE: i32 = 1;
/// The size of a chunk, including a border that will not be rendered. This is to ensure that the
/// `TileType`s of outer tiles are known. Must not be modified directly. Change `CHUNK_SIZE` instead.
pub const CHUNK_SIZE_PLUS_BUFFER: i32 = CHUNK_SIZE + 2 * BUFFER_SIZE;
/// The size of a chunk that is rendered on the screen. This is the size of a chunk minus the
/// border.
pub const CHUNK_SIZE: i32 = 16;
pub const ORIGIN_WORLD_GRID_SPAWN_POINT: Point<WorldGrid> = Point::new_const(-(CHUNK_SIZE / 2), CHUNK_SIZE / 2);
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
// Sprites: Default tile set
pub const TILE_SET_DEFAULT_PATH: &str = "tilesets/default.png";
pub const TILE_SET_DEFAULT_COLUMNS: u32 = 5;
pub const TILE_SET_DEFAULT_ROWS: u32 = 1;
// ------------------------------------------------------------------------------------------------------
// Sprites: Detailed static tile sets
pub const STATIC_TILE_SET_WATER_PATH: &str = "tilesets/water.png";
pub const STATIC_TILE_SET_SHORE_PATH: &str = "tilesets/shore.png";
pub const STATIC_TILE_SET_SAND_PATH: &str = "tilesets/sand.png";
pub const STATIC_TILE_SET_GRASS_PATH: &str = "tilesets/grass.png";
pub const STATIC_TILE_SET_FOREST_PATH: &str = "tilesets/forest.png";
pub const STATIC_TILE_SET_COLUMNS: u32 = 9;
pub const STATIC_TILE_SET_ROWS: u32 = 3;
// ------------------------------------------------------------------------------------------------------
// Sprites: Static tile set sprite indices
pub const STATIC_FILL: usize = 4;
pub const STATIC_INNER_CORNER_BOTTOM_LEFT: usize = 2;
pub const STATIC_INNER_CORNER_BOTTOM_RIGHT: usize = 0;
pub const STATIC_INNER_CORNER_TOP_LEFT: usize = 11;
pub const STATIC_INNER_CORNER_TOP_RIGHT: usize = 9;
pub const STATIC_OUTER_CORNER_BOTTOM_LEFT: usize = 13;
pub const STATIC_OUTER_CORNER_BOTTOM_RIGHT: usize = 12;
pub const STATIC_OUTER_CORNER_TOP_LEFT: usize = 15;
pub const STATIC_OUTER_CORNER_TOP_RIGHT: usize = 14;
pub const STATIC_TOP_LEFT_TO_BOTTOM_RIGHT_BRIDGE: usize = 16;
pub const STATIC_TOP_RIGHT_TO_BOTTOM_LEFT_BRIDGE: usize = 17;
pub const STATIC_TOP_FILL: usize = 10;
pub const STATIC_BOTTOM_FILL: usize = 1;
pub const STATIC_RIGHT_FILL: usize = 3;
pub const STATIC_LEFT_FILL: usize = 5;
pub const STATIC_SINGLE: usize = 18;
pub const STATIC_ERROR: usize = 26;
// ------------------------------------------------------------------------------------------------------
// Sprites: Detailed animated tile sets
pub const ANIMATED_TILE_SET_SHORE_PATH: &str = "tilesets/shore-animated.png";
pub const ANIMATED_TILE_SET_COLUMNS: u32 = 4;
pub const ANIMATED_TILE_SET_ROWS: u32 = 17;
// ------------------------------------------------------------------------------------------------------
// Sprites: Animated tile set sprite indices
pub const ANIMATED_FILL: usize = 4;
pub const ANIMATED_INNER_CORNER_BOTTOM_LEFT: usize = 2;
pub const ANIMATED_INNER_CORNER_BOTTOM_RIGHT: usize = 0;
pub const ANIMATED_INNER_CORNER_TOP_LEFT: usize = 6;
pub const ANIMATED_INNER_CORNER_TOP_RIGHT: usize = 8;
pub const ANIMATED_OUTER_CORNER_BOTTOM_LEFT: usize = 9;
pub const ANIMATED_OUTER_CORNER_BOTTOM_RIGHT: usize = 10;
pub const ANIMATED_OUTER_CORNER_TOP_LEFT: usize = 12;
pub const ANIMATED_OUTER_CORNER_TOP_RIGHT: usize = 11;
pub const ANIMATED_TOP_LEFT_TO_BOTTOM_RIGHT_BRIDGE: usize = 13;
pub const ANIMATED_TOP_RIGHT_TO_BOTTOM_LEFT_BRIDGE: usize = 14;
pub const ANIMATED_TOP_FILL: usize = 7;
pub const ANIMATED_BOTTOM_FILL: usize = 1;
pub const ANIMATED_RIGHT_FILL: usize = 3;
pub const ANIMATED_LEFT_FILL: usize = 5;
pub const ANIMATED_SINGLE: usize = 15;
pub const ANIMATED_ERROR: usize = 16;
// ------------------------------------------------------------------------------------------------------
// Objects
pub const TREES_PATH: &str = "objects/trees-conifer.png";
pub const TREES_COLUMNS: u32 = 5;
pub const TREES_ROWS: u32 = 1;
pub const TREE_SIZE: UVec2 = UVec2::new(64, 128);
// ------------------------------------------------------------------------------------------------------
// Colours
pub const RED: Color = Color::hsl(0.59, 0.32, 0.52);
pub const PURPLE: Color = Color::srgb(0.706, 0.557, 0.678);
pub const YELLOW: Color = Color::srgb(0.922, 0.796, 0.545);
pub const ORANGE: Color = Color::srgb(0.816, 0.529, 0.439);
pub const GREEN: Color = Color::srgb(0.639, 0.745, 0.549);
pub const WATER_BLUE: Color = Color::srgb(0.305882, 0.611765, 0.74902);
pub const LIGHT: Color = Color::srgb(0.925, 0.937, 0.957);
pub const MEDIUM: Color = Color::srgb(0.60, 0.639, 0.714);
pub const DARK: Color = Color::srgb(0.298, 0.337, 0.416);
pub const VERY_DARK: Color = Color::srgb(0.12, 0.14, 0.18);
// ------------------------------------------------------------------------------------------------------
// Window
pub const WINDOW_WIDTH: f32 = 1280.;
pub const WINDOW_HEIGHT: f32 = 720.;
