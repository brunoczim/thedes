use crate::geometry::CoordPair;

pub(crate) mod native_ext;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyEvent {
    pub main_key: Key,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
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
