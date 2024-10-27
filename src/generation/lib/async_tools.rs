#![allow(dead_code)]

use bevy::log::{error, info};
use std::thread;

pub fn log_current_thread(fn_name: &str) {
  let thread = thread::current();
  let thread_name = thread.name().unwrap_or("Unnamed");
  let thread_id = thread.id();
  error!("Executed [{}] on thread: {} (ID: {:?})", fn_name, thread_name, thread_id);
}

pub fn log_with_thread_prefix(message: &str) {
  let thread = thread::current();
  let thread_name = thread.name().unwrap_or("Unnamed");
  let thread_id = thread.id();
  info!("[Thread: {} #{:?} ]: {}", thread_name, thread_id, message);
}

pub fn get_thread_info() -> String {
  let thread = thread::current();
  let thread_name = thread.name().unwrap_or("Unnamed");
  let thread_id = thread.id();
  format!("[{} {:?}]", thread_name, thread_id)
}
