#![allow(dead_code)]

use bevy::color::Color;

// Window
pub const GRID_COLS: usize = 1000;
pub const GRID_ROWS: usize = 800;
pub const GRID_W: usize = GRID_COLS * TILE_WIDTH;
pub const GRID_H: usize = GRID_ROWS * TILE_HEIGHT;
pub const WINDOW_WIDTH: f32 = 1280.;
pub const WINDOW_HEIGHT: f32 = 720.;
pub const BG_COLOR: (u8, u8, u8) = (181, 212, 220);

// Sprites
pub const TILE_SIZE: u32 = 32;
const TILE_WIDTH: usize = 32;
const TILE_HEIGHT: usize = 32;
pub const TILE_SET_DEFAULT_PATH: &str = "tilesets/default.png";
pub const TILE_SET_DEFAULT_COLUMNS: u32 = 9;
pub const TILE_SET_DEFAULT_ROWS: u32 = 3;
pub const TILE_SET_TEST_PATH: &str = "tilesets/test_tileset.png";
pub const TILE_SET_TEST_COLUMNS: u32 = 4;
pub const TILE_SET_TEST_ROWS: u32 = 1;

pub const WATER_TILE: usize = 0;
pub const SAND_TILE: usize = 1;
pub const GRASS_TILE: usize = 2;
pub const FOREST_TILE: usize = 3;

// Chunks
pub const CHUNK_SIZE: i32 = 32;

// Colours
pub(crate) const RED: Color = Color::hsl(0.59, 0.32, 0.52);
pub(crate) const PURPLE: Color = Color::srgb(0.706, 0.557, 0.678);
pub(crate) const YELLOW: Color = Color::srgb(0.922, 0.796, 0.545);
pub(crate) const BLUE: Color = Color::srgb(0.533, 0.753, 0.816);
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

// Fonts
pub(crate) const DEFAULT_FONT: &str = "fonts/bulkypix.ttf";
