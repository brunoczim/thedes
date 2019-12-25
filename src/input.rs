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
}
