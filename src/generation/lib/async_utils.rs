#![allow(dead_code)]

use std::thread;

pub fn get_thread_info() -> String {
  let thread = thread::current();
  let thread_name = thread.name().unwrap_or("Unnamed");
  let thread_id = thread.id();
  format!("[{} {:?}]", thread_name, thread_id)
}
