#![allow(dead_code)]

use bevy::color::Color;

// Animation
pub const BASE_DELAY: f32 = 0.; // 0.0025
pub const LAYER_DELAY: f32 = 2.;

// Settings
pub const DRAW_SPRITES: bool = false;
pub const PERMIT_TILE_LAYER_ADJUSTMENTS: bool = false;

// Window
pub const GRID_COLS: usize = 1000;
pub const GRID_ROWS: usize = 800;
pub const GRID_W: usize = GRID_COLS * TILE_WIDTH;
pub const GRID_H: usize = GRID_ROWS * TILE_HEIGHT;
pub const WINDOW_WIDTH: f32 = 1280.;
pub const WINDOW_HEIGHT: f32 = 720.;
pub const BG_COLOR: (u8, u8, u8) = (181, 212, 220);

// Chunks
pub const CHUNK_SIZE: i32 = 32;

// Sprites
pub const TILE_SIZE: u32 = 32;
const TILE_WIDTH: usize = 32;
const TILE_HEIGHT: usize = 32;

// Default tile set
pub const TILE_SET_DEFAULT_PATH: &str = "tilesets/default.png";
pub const TILE_SET_DEFAULT_COLUMNS: u32 = 5;
pub const TILE_SET_DEFAULT_ROWS: u32 = 1;

// Default tiles
pub const WATER_TILE: usize = 0;
pub const SHORE_TILE: usize = 1;
pub const SAND_TILE: usize = 2;
pub const GRASS_TILE: usize = 3;
pub const FOREST_TILE: usize = 4;

// Detailed tile sets
pub const TILE_SET_SAND_PATH: &str = "tilesets/sand.png";
pub const TILE_SET_GRASS_PATH: &str = "tilesets/grass.png";
pub const TILE_SET_COLUMNS: u32 = 9;
pub const TILE_SET_ROWS: u32 = 3;

// Detailed tiles
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
pub const ERROR: usize = 26;

// Colours
pub(crate) const RED: Color = Color::hsl(0.59, 0.32, 0.52);
pub(crate) const PURPLE: Color = Color::srgb(0.706, 0.557, 0.678);
pub(crate) const YELLOW: Color = Color::srgb(0.922, 0.796, 0.545);
pub(crate) const ORANGE: Color = Color::srgb(0.816, 0.529, 0.439);
pub(crate) const GREEN: Color = Color::srgb(0.639, 0.745, 0.549);
pub(crate) const LIGHT_1: Color = Color::srgb(0.925, 0.937, 0.957);
pub(crate) const LIGHT_2: Color = Color::srgb(0.898, 0.914, 0.941);
pub(crate) const LIGHT_3: Color = Color::srgb(0.847, 0.871, 0.914);
pub(crate) const MEDIUM_1: Color = Color::srgb(0.60, 0.639, 0.714);
pub(crate) const MEDIUM_2: Color = Color::srgb(0.427, 0.478, 0.588);
pub(crate) const DARK_1: Color = Color::srgb(0.298, 0.337, 0.416);
pub(crate) const DARK_4: Color = Color::srgb(0.18, 0.204, 0.251);
pub(crate) const VERY_DARK_1: Color = Color::srgb(0.12, 0.14, 0.18);
pub(crate) const VERY_DARK_2: Color = Color::srgb(0.06, 0.07, 0.09);

pub(crate) fn get_blue() -> Color {
  Color::srgba_u8(100, 183, 220, 255)
}

// Fonts
pub(crate) const DEFAULT_FONT: &str = "fonts/bulkypix.ttf";
