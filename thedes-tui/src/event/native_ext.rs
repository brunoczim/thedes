use crossterm::event::{
    Event as CrosstermEvent,
    KeyCode as CrosstermKey,
    KeyEvent as CrosstermKeyEvent,
    KeyModifiers as CrosstermKeyModifiers,
};

use crate::geometry::CoordPair;

use super::{Event, InternalEvent, Key, KeyEvent, PasteEvent, ResizeEvent};

pub(crate) trait FromCrossterm<E>: Sized {
    fn from_crossterm(crossterm: E) -> Option<Self>;
}

impl FromCrossterm<CrosstermKey> for Key {
    fn from_crossterm(crossterm: CrosstermKey) -> Option<Self> {
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

impl FromCrossterm<CrosstermKeyEvent> for KeyEvent {
    fn from_crossterm(crossterm_event: CrosstermKeyEvent) -> Option<Self> {
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

impl FromCrossterm<CrosstermEvent> for InternalEvent {
    fn from_crossterm(crossterm_event: CrosstermEvent) -> Option<Self> {
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
