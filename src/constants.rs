#![allow(dead_code)]

use bevy::color::Color;

// Animation
pub const BASE_DELAY: f32 = 0.; // 0.0025
pub const LAYER_DELAY: f32 = 2.;

// Settings
pub const SHOW_SETTINGS_UI: bool = true;
pub const GENERATE_NEIGHBOUR_CHUNKS: bool = false;
pub const ENABLE_TILE_DEBUGGING: bool = true;
pub const DRAW_TERRAIN_SPRITES: bool = true;
pub const LAYER_POST_PROCESSING: bool = false;
pub const SPAWN_UP_TO_LAYER: usize = 5;

// Chunks
pub const CHUNK_SIZE: i32 = 32;

// Sprites
pub const TILE_SIZE: u32 = 32;
const TILE_WIDTH: usize = 32;
const TILE_HEIGHT: usize = 32;

// Layers
pub const WATER_LAYER: usize = 0;
pub const SHORE_LAYER: usize = 1;
pub const SAND_LAYER: usize = 2;
pub const GRASS_LAYER: usize = 3;
pub const FOREST_LAYER: usize = 4;

// Default tile set
pub const TILE_SET_DEFAULT_PATH: &str = "tilesets/default.png";
pub const TILE_SET_DEFAULT_COLUMNS: u32 = 5;
pub const TILE_SET_DEFAULT_ROWS: u32 = 1;

// Detailed tile sets
pub const TILE_SET_WATER_PATH: &str = "tilesets/water.png";
pub const TILE_SET_SHORE_PATH: &str = "tilesets/shore.png";
pub const TILE_SET_SAND_PATH: &str = "tilesets/sand.png";
pub const TILE_SET_GRASS_PATH: &str = "tilesets/grass.png";
pub const TILE_SET_FOREST_PATH: &str = "tilesets/forest.png";
pub const TILE_SET_COLUMNS: u32 = 9;
pub const TILE_SET_ROWS: u32 = 3;

// Tile set sprite indices
pub const FILL: usize = 4;
pub const INNER_CORNER_BOTTOM_LEFT: usize = 2;
pub const INNER_CORNER_BOTTOM_RIGHT: usize = 0;
pub const INNER_CORNER_TOP_LEFT: usize = 11;
pub const INNER_CORNER_TOP_RIGHT: usize = 9;
pub const OUTER_CORNER_BOTTOM_LEFT: usize = 13;
pub const OUTER_CORNER_BOTTOM_RIGHT: usize = 12;
pub const OUTER_CORNER_TOP_LEFT: usize = 15;
pub const OUTER_CORNER_TOP_RIGHT: usize = 14;
pub const TOP_LEFT_TO_BOTTOM_RIGHT_BRIDGE: usize = 16;
pub const TOP_RIGHT_TO_BOTTOM_LEFT_BRIDGE: usize = 17;
pub const TOP_FILL: usize = 10;
pub const BOTTOM_FILL: usize = 1;
pub const RIGHT_FILL: usize = 3;
pub const LEFT_FILL: usize = 5;
pub const SINGLE: usize = 18;
pub const ERROR: usize = 26;

// Colours
pub(crate) const RED: Color = Color::hsl(0.59, 0.32, 0.52);
pub(crate) const PURPLE: Color = Color::srgb(0.706, 0.557, 0.678);
pub(crate) const YELLOW: Color = Color::srgb(0.922, 0.796, 0.545);
pub(crate) const ORANGE: Color = Color::srgb(0.816, 0.529, 0.439);
pub(crate) const GREEN: Color = Color::srgb(0.639, 0.745, 0.549);
pub(crate) const WATER_BLUE: Color = Color::srgb(0.305882, 0.611765, 0.74902);
pub(crate) const LIGHT_1: Color = Color::srgb(0.925, 0.937, 0.957);
pub(crate) const LIGHT_2: Color = Color::srgb(0.898, 0.914, 0.941);
pub(crate) const LIGHT_3: Color = Color::srgb(0.847, 0.871, 0.914);
pub(crate) const MEDIUM_1: Color = Color::srgb(0.60, 0.639, 0.714);
pub(crate) const MEDIUM_2: Color = Color::srgb(0.427, 0.478, 0.588);
pub(crate) const DARK_1: Color = Color::srgb(0.298, 0.337, 0.416);
pub(crate) const DARK_4: Color = Color::srgb(0.18, 0.204, 0.251);
pub(crate) const VERY_DARK_1: Color = Color::srgb(0.12, 0.14, 0.18);
pub(crate) const VERY_DARK_2: Color = Color::srgb(0.06, 0.07, 0.09);

// Fonts
pub(crate) const DEFAULT_FONT: &str = "fonts/Minimal5x7.ttf";
pub(crate) const MINIMAL_5X5_MONO_FONT: &str = "fonts/Minimal5x5Monospaced.ttf";
pub(crate) const BULKYPIX_FONT: &str = "fonts/bulkypix.ttf";

// Window
pub const WINDOW_WIDTH: f32 = 1280.;
pub const WINDOW_HEIGHT: f32 = 720.;