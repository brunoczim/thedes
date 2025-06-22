use std::fmt;

use thedes_tui::{
    cancellability::Cancellable,
    core::App,
    info::{self, Info},
    input::{self, Input},
    menu::{self, Menu},
};
use thiserror::Error;

#[derive(Debug, Error)]
#[error("Truncated {} characters from input", .0)]
pub struct SetNameError(usize);

#[derive(Debug, Error)]
pub enum InitError {
    #[error("Failed to initialize menu")]
    Menu(
        #[source]
        #[from]
        menu::Error,
    ),
    #[error("Failed to create name input")]
    Name(#[source] input::Error),
    #[error("Failed to create seed input")]
    Seed(#[source] input::Error),
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to run menu")]
    RunMenu(
        #[source]
        #[from]
        menu::Error,
    ),
    #[error("Failed to run name input")]
    RunName(#[source] input::Error),
    #[error("Failed to run seed input")]
    RunSeed(#[source] input::Error),
    #[error("Failed to display information regarding name input")]
    EmptyNameInfo(#[source] info::Error),
    #[error("Failed to display information regarding seed input")]
    EmptySeedInfo(#[source] info::Error),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Form {
    pub name: String,
    pub seed: Seed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NewGameMenuItem {
    Create,
    SetName,
    SetSeed,
}

impl fmt::Display for NewGameMenuItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Create => "Create",
            Self::SetName => "Set Name",
            Self::SetSeed => "Set Seed",
        })
    }
}

pub type Seed = u32;

#[derive(Debug, Clone)]
pub struct Component {
    form: Form,
    menu: Menu<NewGameMenuItem, Cancellable>,
    name_input: Input<fn(char) -> bool, Cancellable>,
    seed_input: Input<fn(char) -> bool, Cancellable>,
    empty_name_info: Info,
    empty_seed_info: Info,
}

impl Component {
    pub fn new() -> Result<Self, InitError> {
        let menu = Menu::from_cancellation(
            "New Game",
            [
                NewGameMenuItem::Create,
                NewGameMenuItem::SetName,
                NewGameMenuItem::SetSeed,
            ],
            Cancellable::new(false),
        )?;

        let form = Form::default();

        let result = Input::from_cancellation(
            input::Config {
                max: 32,
                title: "New Game's Name",
                filter: (|ch| {
                    ch.is_ascii_alphanumeric() || ch == '_' || ch == '-'
                }) as fn(char) -> bool,
            },
            Cancellable::new(false),
        );
        let name_input = result.map_err(InitError::Name)?;

        let result = Input::from_cancellation(
            input::Config {
                max: 8,
                title: "New Game's Seed",
                filter: (|ch| ch.is_ascii_hexdigit()) as fn(char) -> bool,
            },
            Cancellable::new(false),
        );
        let seed_input = result.map_err(InitError::Seed)?;

        let empty_name_info = Info::new("Error!", "Game name cannot be empty");
        let empty_seed_info = Info::new("Error!", "Game seed cannot be empty");

        Ok(Self {
            menu,
            form,
            name_input,
            seed_input,
            empty_name_info,
            empty_seed_info,
        })
    }

    pub fn set_seed(&mut self, seed: Seed) -> &mut Self {
        self.form.seed = seed;
        self
    }

    pub fn set_name(
        &mut self,
        name: impl Into<String>,
    ) -> Result<&mut Self, SetNameError> {
        let name = name.into();
        let input_max = usize::from(self.name_input.max());
        if name.len() > input_max {
            Err(SetNameError(name.len() - input_max))?
        }
        self.form.name = name;
        Ok(self)
    }

    pub fn form(&self) -> &Form {
        &self.form
    }

    pub fn is_cancelling(&self) -> bool {
        self.menu.is_cancelling() || self.form.name.is_empty()
    }

    pub fn output(&self) -> Option<&Form> {
        if self.is_cancelling() {
            None?
        }
        Some(self.form())
    }

    fn load_form(&mut self) {
        let seed_digits = format!("{:x}", self.form.seed);
        let _ = self.seed_input.set_buffer(seed_digits.chars());
        let _ = self.name_input.set_buffer(self.form.name.chars());
    }

    pub async fn run(&mut self, app: &mut App) -> Result<(), Error> {
        self.load_form();
        self.read_name(app).await?;
        if self.name_input.is_cancelling() {
            self.menu.set_cancelling(true);
        } else {
            self.menu.set_selected(0)?;
            self.menu.set_cancelling(false);

            loop {
                self.menu.run(app).await?;
                match self.menu.output() {
                    Some(NewGameMenuItem::Create) => break,
                    Some(NewGameMenuItem::SetName) => {
                        self.read_name(app).await?;
                    },
                    Some(NewGameMenuItem::SetSeed) => {
                        self.read_seed(app).await?;
                    },
                    None => break,
                }
            }
        }
        Ok(())
    }

    async fn read_name(&mut self, app: &mut App) -> Result<(), Error> {
        loop {
            self.name_input.run(app).await.map_err(Error::RunName)?;
            let name = self.name_input.output();

            match name {
                Some(name) if name.is_empty() => {
                    self.empty_name_info
                        .run(app)
                        .await
                        .map_err(Error::EmptyNameInfo)?;
                },
                Some(name) => {
                    self.form.name = name;
                    break;
                },
                None => break,
            }
        }
        Ok(())
    }

    async fn read_seed(&mut self, app: &mut App) -> Result<(), Error> {
        loop {
            self.seed_input.run(app).await.map_err(Error::RunSeed)?;
            let seed = self.seed_input.output();

            match seed.map(|s| Seed::from_str_radix(&s, 16)) {
                Some(Err(_)) => {
                    self.empty_seed_info
                        .run(app)
                        .await
                        .map_err(Error::EmptySeedInfo)?;
                },
                Some(Ok(seed)) => {
                    self.form.seed = seed;
                    break;
                },
                None => break,
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use thedes_tui::core::{
        App,
        event::Key,
        geometry::CoordPair,
        runtime::{Config, device::mock::RuntimeDeviceMock},
        screen,
    };
    use thiserror::Error;
    use tokio::{task, time::timeout};

    #[derive(Debug, Error)]
    enum Error {
        #[error(transparent)]
        Init(#[from] super::InitError),
        #[error(transparent)]
        Run(#[from] super::Error),
    }

    async fn tui_main(mut app: App) -> Result<Option<super::Form>, Error> {
        let mut component = super::Component::new()?;
        component.run(&mut app).await?;
        Ok(component.output().cloned())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn cancel_first_name_input() {
        let device_mock = RuntimeDeviceMock::new(CoordPair { y: 24, x: 80 });
        let device = device_mock.open();
        let config = Config::new()
            .with_screen(
                screen::Config::new()
                    .with_canvas_size(CoordPair { y: 22, x: 78 }),
            )
            .with_device(device);

        device_mock.input().publish_ok([Key::Esc]);

        let runtime_future = task::spawn(config.run(tui_main));
        let output = timeout(Duration::from_millis(200), runtime_future)
            .await
            .unwrap()
            .unwrap()
            .unwrap()
            .unwrap();
        assert_eq!(output, None);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn confirm_default_seed() {
        let device_mock = RuntimeDeviceMock::new(CoordPair { y: 24, x: 80 });
        let device = device_mock.open();
        let config = Config::new()
            .with_screen(
                screen::Config::new()
                    .with_canvas_size(CoordPair { y: 22, x: 78 }),
            )
            .with_device(device);

        device_mock.input().publish_ok([
            Key::Char('w'),
            Key::Char('0'),
            Key::Enter,
        ]);

        let runtime_future = task::spawn(config.run(tui_main));
        tokio::time::sleep(Duration::from_millis(50)).await;
        device_mock.input().publish_ok([Key::Enter]);

        let output = timeout(Duration::from_millis(200), runtime_future)
            .await
            .unwrap()
            .unwrap()
            .unwrap()
            .unwrap();
        assert_eq!(
            output,
            Some(super::Form { name: "w0".to_owned(), seed: 0 })
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn cancel_menu() {
        let device_mock = RuntimeDeviceMock::new(CoordPair { y: 24, x: 80 });
        let device = device_mock.open();
        let config = Config::new()
            .with_screen(
                screen::Config::new()
                    .with_canvas_size(CoordPair { y: 22, x: 78 }),
            )
            .with_device(device);

        device_mock.input().publish_ok([
            Key::Char('w'),
            Key::Char('0'),
            Key::Enter,
        ]);

        let runtime_future = task::spawn(config.run(tui_main));
        tokio::time::sleep(Duration::from_millis(50)).await;
        device_mock.input().publish_ok([Key::Esc]);

        let output = timeout(Duration::from_millis(200), runtime_future)
            .await
            .unwrap()
            .unwrap()
            .unwrap()
            .unwrap();
        assert_eq!(output, None);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn confirm_custom_seed() {
        let device_mock = RuntimeDeviceMock::new(CoordPair { y: 24, x: 80 });
        let device = device_mock.open();
        let config = Config::new()
            .with_screen(
                screen::Config::new()
                    .with_canvas_size(CoordPair { y: 22, x: 78 }),
            )
            .with_device(device);

        device_mock.input().publish_ok([
            Key::Char('w'),
            Key::Char('0'),
            Key::Enter,
        ]);

        let runtime_future = task::spawn(config.run(tui_main));

        tokio::time::sleep(Duration::from_millis(50)).await;
        device_mock.input().publish_ok([Key::Down, Key::Down, Key::Enter]);
        tokio::time::sleep(Duration::from_millis(50)).await;
        device_mock.input().publish_ok([
            Key::Char('5'),
            Key::Char('a'),
            Key::Char('9'),
            Key::Enter,
        ]);

        tokio::time::sleep(Duration::from_millis(50)).await;
        device_mock.input().publish_ok([Key::Up, Key::Up, Key::Enter]);

        let output = timeout(Duration::from_millis(200), runtime_future)
            .await
            .unwrap()
            .unwrap()
            .unwrap()
            .unwrap();
        assert_eq!(
            output,
            Some(super::Form { name: "w0".to_owned(), seed: 0x5a9 })
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn do_not_allow_empty_name() {
        let device_mock = RuntimeDeviceMock::new(CoordPair { y: 24, x: 80 });
        let device = device_mock.open();
        let config = Config::new()
            .with_screen(
                screen::Config::new()
                    .with_canvas_size(CoordPair { y: 22, x: 78 }),
            )
            .with_device(device);

        let runtime_future = task::spawn(config.run(tui_main));
        tokio::time::sleep(Duration::from_millis(50)).await;

        device_mock.input().publish_ok([Key::Enter]);
        tokio::time::sleep(Duration::from_millis(50)).await;

        device_mock.input().publish_ok([Key::Enter]);
        tokio::time::sleep(Duration::from_millis(50)).await;

        device_mock.input().publish_ok([
            Key::Char('w'),
            Key::Char('0'),
            Key::Enter,
        ]);
        tokio::time::sleep(Duration::from_millis(50)).await;

        device_mock.input().publish_ok([Key::Enter]);

        let output = timeout(Duration::from_millis(200), runtime_future)
            .await
            .unwrap()
            .unwrap()
            .unwrap()
            .unwrap();
        assert_eq!(
            output,
            Some(super::Form { name: "w0".to_owned(), seed: 0 })
        );
    }
}
