use std::error::Error;

use thedes_domain::game::Game;
use thedes_tui::{
    color::BasicColor,
    event::{Event, Key, KeyEvent},
    TextStyle,
    Tick,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TickError {
    #[error(transparent)]
    Render(#[from] thedes_tui::CanvasError),
    #[error("Failed to run command(s)")]
    Run(
        #[from]
        #[source]
        thedes_dev::Error,
    ),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Action {
    Run(char),
    RunPrevious,
    Exit,
}

#[derive(Debug, Clone)]
pub struct Component {
    previous: char,
}

impl Component {
    pub const DEFAULT_KEY: char = '.';

    pub fn new() -> Self {
        Self { previous: Self::DEFAULT_KEY }
    }

    pub fn reset(&mut self) {
        self.previous = Self::DEFAULT_KEY;
    }

    pub fn on_tick(
        &mut self,
        tick: &mut Tick,
        game: &mut Game,
    ) -> Result<bool, TickError> {
        self.render(tick)?;
        match self.handle_events(tick) {
            Some(Action::Run(ch)) => {
                self.run(ch, game);
                Ok(false)
            },
            Some(Action::RunPrevious) => {
                self.run_previous(game);
                Ok(false)
            },
            Some(Action::Exit) => Ok(false),
            None => Ok(true),
        }
    }

    pub fn run_previous(&mut self, game: &mut Game) {
        self.run(self.previous, game);
    }

    fn run(&mut self, ch: char, game: &mut Game) {
        self.previous = ch;
        if let Err(error) = thedes_dev::run(ch, game) {
            tracing::error!("Failed running development script: {}", error);
            tracing::warn!("Caused by:");
            let mut source = error.source();
            while let Some(current) = source {
                tracing::warn!("- {}", current);
                source = current.source();
            }
        }
    }

    fn render(&mut self, tick: &mut Tick) -> Result<(), TickError> {
        tick.screen_mut().clear_canvas(BasicColor::Black.into())?;
        tick.screen_mut().styled_text(
            "Development Script Mode",
            &TextStyle::default().with_top_margin(1).with_align(1, 2),
        )?;
        tick.screen_mut().styled_text(
            "Press any character key to run a corresponding script.",
            &TextStyle::default().with_top_margin(4).with_align(1, 2),
        )?;
        tick.screen_mut().styled_text(
            "Press enter to run previous script or the default one.",
            &TextStyle::default().with_top_margin(4).with_align(1, 2),
        )?;
        tick.screen_mut().styled_text(
            "Press ESC to cancel.",
            &TextStyle::default().with_top_margin(6).with_align(1, 2),
        )?;
        Ok(())
    }

    fn handle_events(&mut self, tick: &mut Tick) -> Option<Action> {
        while let Some(event) = tick.next_event() {
            match event {
                Event::Key(KeyEvent { main_key: Key::Char(ch), .. }) => {
                    return Some(Action::Run(ch))
                },

                Event::Key(KeyEvent { main_key: Key::Enter, .. }) => {
                    return Some(Action::RunPrevious)
                },

                Event::Key(KeyEvent { main_key: Key::Esc, .. }) => {
                    return Some(Action::Exit)
                },

                _ => (),
            }
        }
        None
    }
}
