use thedes_dev::ScriptTable;
use thedes_domain::game::Game;
use thedes_tui::{
    core::{
        App,
        color::BasicColor,
        event::{Event, Key, KeyEvent},
        screen::{self, FlushError},
    },
    text,
};
use thiserror::Error;

pub type KeyBindingMap = thedes_tui::key_bindings::KeyBindingMap<Command>;

pub fn default_key_bindings() -> KeyBindingMap {
    KeyBindingMap::new()
        .with(Key::Esc, Command::Exit)
        .with(Key::Enter, Command::RunPrevious)
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to render message")]
    RenderMessage(
        #[source]
        #[from]
        text::Error,
    ),
    #[error("TUI cancelled")]
    Cancelled,
    #[error("Failed to run the script")]
    Script(
        #[from]
        #[source]
        thedes_dev::Error,
    ),
    #[error("Failed to flush canvas")]
    FlushCanvas(
        #[from]
        #[source]
        FlushError,
    ),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Command {
    Run(char),
    RunPrevious,
    Exit,
}

#[derive(Debug, Clone)]
pub struct Component {
    prev: char,
    key_bindings: KeyBindingMap,
}

impl Component {
    pub const DEFAULT_KEY: char = '.';

    pub fn new() -> Self {
        Self { prev: Self::DEFAULT_KEY, key_bindings: default_key_bindings() }
    }

    pub fn with_keybindings(mut self, map: KeyBindingMap) -> Self {
        self.set_keybindings(map);
        self
    }

    pub fn set_keybindings(&mut self, map: KeyBindingMap) -> &mut Self {
        self.key_bindings = map;
        self
    }

    pub async fn run(
        &mut self,
        app: &mut App,
        game: &mut Game,
    ) -> Result<(), Error> {
        loop {
            match self.handle_input(app, game).await {
                Ok(false) => break,
                Ok(true) => (),
                Err(Error::Script(e))
                    if matches!(
                        e.kind(),
                        thedes_dev::ErrorKind::UnknownKey(_)
                    ) =>
                {
                    ()
                },
                Err(e) => Err(e)?,
            }
            self.render(app)?;
            tokio::select! {
                _ = app.tick_session.tick() => (),
                _ = app.cancel_token.cancelled() => Err(Error::Cancelled)?,
            }
        }
        Ok(())
    }

    fn render(&mut self, app: &mut App) -> Result<(), Error> {
        app.canvas
            .queue([screen::Command::ClearScreen(BasicColor::Black.into())]);
        text::styled(
            app,
            "Development Script Mode",
            &text::Style::default().with_top_margin(1).with_align(1, 2),
        )?;
        text::styled(
            app,
            "Press any character key to run a corresponding script.",
            &text::Style::default().with_top_margin(4).with_align(1, 2),
        )?;
        text::styled(
            app,
            "Press enter to run previous script or the default one.",
            &text::Style::default().with_top_margin(5).with_align(1, 2),
        )?;
        text::styled(
            app,
            "Press ESC to cancel.",
            &text::Style::default().with_top_margin(7).with_align(1, 2),
        )?;
        app.canvas.flush()?;
        Ok(())
    }

    async fn handle_input(
        &mut self,
        app: &mut App,
        game: &mut Game,
    ) -> Result<bool, Error> {
        let Ok(mut events) = app.events.read_until_now() else {
            Err(Error::Cancelled)?
        };

        let mut should_continue = true;
        let mut hit = false;
        while let Some(event) = events.next().filter(|_| should_continue) {
            let Event::Key(key) = event else { continue };

            let command = self.key_bindings.command_for(key).cloned();

            should_continue = match command {
                Some(command) => {
                    hit = true;
                    self.run_command(game, command).await?
                },
                None => match key {
                    KeyEvent {
                        main_key: Key::Char(ch),
                        ctrl: false,
                        alt: false,
                        shift: false,
                    } => {
                        hit = true;
                        self.run_command(game, Command::Run(ch)).await?
                    },
                    _ => true,
                },
            };
        }
        Ok(!hit)
    }

    async fn run_command(
        &mut self,
        game: &mut Game,
        command: Command,
    ) -> Result<bool, Error> {
        match command {
            Command::Run(ch) => ScriptTable::run_reading(ch, game).await?,
            Command::RunPrevious => {
                ScriptTable::run_reading(self.prev, game).await?
            },
            Command::Exit => return Ok(false),
        }
        Ok(true)
    }
}
