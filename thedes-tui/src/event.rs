use crate::geometry::CoordPair;
use crossterm::event::{
    Event as CrosstermEvent,
    KeyCode as CrosstermKey,
    KeyEvent as CrosstermKeyEvent,
    KeyModifiers as CrosstermKeyModifiers,
};

/// A supported pressed key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    /// A regular, unicode character. E.g. `Key::Char('a')` or
    /// `Key::Char('ç')`.
    Char(char),
    /// The up arrow key.
    Up,
    /// The down arrow key.
    Down,
    /// The left arrow key.
    Left,
    /// The right arrow key.
    Right,
    /// The escape key.
    Esc,
    /// The enter key. Preferred over `Char('\n')`.
    Enter,
    /// The backspace key
    Backspace,
    /// The delete key
    Delete,
}

impl Key {
    pub(crate) fn from_crossterm(crossterm: CrosstermKey) -> Option<Self> {
        match crossterm {
            CrosstermKey::Esc => Some(Self::Esc),
            CrosstermKey::Backspace => Some(Self::Backspace),
            CrosstermKey::Delete => Some(Self::Delete),
            CrosstermKey::Enter => Some(Self::Enter),
            CrosstermKey::Up => Some(Self::Up),
            CrosstermKey::Down => Some(Self::Down),
            CrosstermKey::Left => Some(Self::Left),
            CrosstermKey::Right => Some(Self::Right),
            CrosstermKey::Char(ch) => Some(Self::Char(ch)),
            _ => None,
        }
    }
}

/// An event fired by a key pressed by the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyEvent {
    /// Key pressed by the user.
    pub main_key: Key,
    /// Whether control is modifiying the key (pressed).
    pub ctrl: bool,
    /// Whether alt is modifiying the key (pressed).
    pub alt: bool,
    /// Whether shift is modifiying the key (pressed).
    pub shift: bool,
}

impl KeyEvent {
    pub(crate) fn from_crossterm(
        crossterm_event: CrosstermKeyEvent,
    ) -> Option<Self> {
        let key = Key::from_crossterm(crossterm_event.code)?;
        Some(Self {
            main_key: key,
            ctrl: crossterm_event
                .modifiers
                .contains(CrosstermKeyModifiers::CONTROL),
            alt: crossterm_event.modifiers.contains(CrosstermKeyModifiers::ALT),
            shift: crossterm_event
                .modifiers
                .contains(CrosstermKeyModifiers::SHIFT),
        })
    }
}

/// An event fired by the user pasting data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PasteEvent {
    /// Data pasted by the user.
    pub data: String,
}

/// A generic event type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// User pressed key.
    Key(KeyEvent),
    /// User pasted a string.
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

/// An event fired by a resize of the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ResizeEvent {
    pub size: CoordPair,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum InternalEvent {
    External(Event),
    Resize(ResizeEvent),
}

impl InternalEvent {
    pub(crate) fn from_crossterm(
        crossterm_event: CrosstermEvent,
    ) -> Option<Self> {
        match crossterm_event {
            CrosstermEvent::Key(event) => KeyEvent::from_crossterm(event)
                .map(Event::Key)
                .map(Self::External),
            CrosstermEvent::Paste(data) => {
                Some(Self::External(Event::Paste(PasteEvent { data })))
            },
            CrosstermEvent::Resize(x, y) => {
                Some(Self::Resize(ResizeEvent { size: CoordPair { x, y } }))
            },
            _ => None,
        }
    }
}
