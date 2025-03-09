#[cfg(target_family = "wasm")]
pub use crate::wasm::extensions as wasm;

#[cfg(not(target_family = "wasm"))]
pub use crate::native::extensions as native;
