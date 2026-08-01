

use std::path::Path;

use mlua::{Lua, LuaOptions, StdLib};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InitError {
    #[error("Failed to load Lua")]
    Lua(#[from] mlua::Error),
}

#[derive(Debug, Clone)]
pub struct Runtime {
    lua: Lua,
}

impl Runtime {
    pub fn new() -> Result<Self, InitError> {
        let lua = Lua::new_with(
            StdLib::COROUTINE
                | StdLib::INTEGER
                | StdLib::VECTOR
                | StdLib::STRING
                | StdLib::UTF8
                | StdLib::TABLE
                | StdLib::MATH
                | StdLib::BUFFER
                | StdLib::BIT,
            LuaOptions::new().catch_rust_panics(true),
        )?;
        Ok(Self { lua })
    }

    pub fn load_package(&mut self, dir_path: &Path)
}
