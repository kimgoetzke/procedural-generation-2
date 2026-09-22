mod cell;
mod permitted_object_names;
mod tile_below;

pub(in crate::generation::object) use cell::PropagationFailure;
pub use cell::{Cell, CellRef};
