use crate::coordinates::Point;
use crate::coordinates::point::ChunkGrid;
use bevy::color::Color;
use bevy_inspector_egui::egui::Color32;
use std::thread;
use std::time::SystemTime;

pub fn thread_name() -> String {
  let thread = thread::current();
  let thread_name = thread.name().unwrap_or("Unnamed");
  let thread_id = thread.id();

  format!("[{} {:?}]", thread_name, thread_id)
}

pub fn get_time() -> u128 {
  SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()
}

pub const fn calculate_seed(cg: Point<ChunkGrid>, seed: u32) -> u64 {
  let adjusted_x = cg.x as i64 + i32::MAX as i64;
  let adjusted_y = cg.y as i64 + i32::MAX as i64;

  ((adjusted_x as u64) << 32) ^ ((adjusted_y as u64) + seed as u64)
}

pub fn to_colour_32(colour: Color) -> Color32 {
  let colour = colour.to_srgba();

  Color32::from_rgb(
    (colour.red * 255.) as u8,
    (colour.green * 255.) as u8,
    (colour.blue * 255.) as u8,
  )
}
