use std::any::Any;

pub use async_trait::async_trait as dynamic_async_trait;
pub use futures;
pub use trait_variant::make as static_async_trait;

#[cfg(not(target_family = "wasm"))]
use native as backend;

#[cfg(target_family = "wasm")]
use wasm as backend;

#[macro_use]
mod macros;

#[cfg(not(target_family = "wasm"))]
mod native;

#[cfg(target_family = "wasm")]
mod wasm;

pub mod extensions;
pub mod task;
pub mod time;
pub mod sync;
pub mod local;

pub type PanicPayload = Box<dyn Any + Send + 'static>;
