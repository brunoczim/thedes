use std::collections::HashMap;

use thedes_tui_core::event::KeyEvent;

#[derive(Debug, Clone)]
pub struct KeyBindingMap<C> {
    table: HashMap<KeyEvent, C>,
}

impl<C> Default for KeyBindingMap<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C> KeyBindingMap<C> {
    pub fn new() -> Self {
        Self { table: HashMap::new() }
    }

    pub fn command_for(&self, key: impl Into<KeyEvent>) -> Option<&C> {
        self.table.get(&key.into())
    }

    pub fn with(
        mut self,
        key: impl Into<KeyEvent>,
        command: impl Into<C>,
    ) -> Self {
        self.bind(key, command);
        self
    }

    pub fn bind(
        &mut self,
        key: impl Into<KeyEvent>,
        command: impl Into<C>,
    ) -> Option<C> {
        self.table.insert(key.into(), command.into())
    }

    pub fn unbind(&mut self, key: impl Into<KeyEvent>) -> Option<C> {
        self.table.remove(&key.into())
    }
}
