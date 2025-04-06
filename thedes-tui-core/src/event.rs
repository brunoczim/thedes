use crate::geometry::CoordPair;

pub(crate) mod native_ext;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Key {
    Char(char),
    Up,
    Down,
    Left,
    Right,
    Esc,
    Enter,
    Backspace,
    Delete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyEvent {
    pub main_key: Key,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

impl KeyEvent {
    pub fn new(main_key: Key) -> Self {
        Self { main_key, ctrl: false, alt: false, shift: false }
    }

    pub fn with_ctrl(mut self, pressed: bool) -> Self {
        self.ctrl = pressed;
        self
    }

    pub fn with_alt(mut self, pressed: bool) -> Self {
        self.alt = pressed;
        self
    }

    pub fn with_shift(mut self, pressed: bool) -> Self {
        self.shift = pressed;
        self
    }
}

impl From<Key> for KeyEvent {
    fn from(main_key: Key) -> Self {
        Self::new(main_key)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PasteEvent {
    pub data: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Key(KeyEvent),
    Paste(PasteEvent),
}

impl From<KeyEvent> for Event {
    fn from(event: KeyEvent) -> Self {
        Event::Key(event)
    }
}

impl From<PasteEvent> for Event {
    fn from(event: PasteEvent) -> Self {
        Event::Paste(event)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResizeEvent {
    pub size: CoordPair,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InternalEvent {
    External(Event),
    Resize(ResizeEvent),
}
