use crate::{graphics::GString, ui::MenuOption};

#[derive(Debug)]
/// Menu shown when player pauses.
pub enum PauseMenuOption {
    /// User asked to resume game.
    Resume,
    /// User asked to exit game to main menu.
    Exit,
}

impl MenuOption for PauseMenuOption {
    fn name(&self) -> GString {
        let string = match self {
            Self::Resume => "RESUME",
            Self::Exit => "EXIT TO MAIN MENU",
        };

        gstring![string]
    }
}
