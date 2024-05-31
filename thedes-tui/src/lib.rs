pub mod geometry;
pub mod color;
pub mod grapheme;
pub mod tile;
pub mod event;
pub mod style;
pub mod panic;
pub mod component;

mod screen;
mod app;
mod config;

pub use app::{ExecutionError, InitError, Tick};
pub use config::Config;
pub use screen::{RenderError, Screen};
