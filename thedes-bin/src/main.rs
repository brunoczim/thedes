use std::{
    backtrace::Backtrace,
    env,
    error::Error,
    fmt,
    fs,
    io,
    panic,
    path::PathBuf,
    sync::Arc,
};

use chrono::{Datelike, Timelike};
use rand::{thread_rng, RngCore};
use thedes_tui::{
    component::{
        info::{self, InfoDialog},
        input::{self, InputDialog},
        menu::{self, Menu},
        Cancellability,
        Cancellable,
        NonCancellable,
    },
    Tick,
};
use thiserror::Error;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{
    filter::FromEnvError,
    layer::SubscriberExt,
    registry,
    util::SubscriberInitExt,
    EnvFilter,
    Layer,
};

#[derive(Debug, Error)]
enum ProgramError {
    #[error("Failed to open log file {}", .path.display())]
    OpenLogFile {
        path: PathBuf,
        #[source]
        cause: io::Error,
    },
    #[error("Failed to get log filter")]
    LogFilter(
        #[source]
        #[from]
        FromEnvError,
    ),
    #[error("Failed to initialize TUI application")]
    AppInit(
        #[source]
        #[from]
        AppInitError,
    ),
    #[error("Failed to run TUI application")]
    AppExec(
        #[source]
        #[from]
        thedes_tui::ExecutionError<AppExecError>,
    ),
}

#[derive(Debug, Error)]
enum AppInitError {
    #[error("Inconsistency setting maximum seed size")]
    MaxSeedSize(#[source] input::CursorOutOfBounds),
    #[error("Inconsistency setting maximum save name size")]
    MaxSaveNameSize(#[source] input::CursorOutOfBounds),
}

#[derive(Debug, Error)]
enum AppExecError {
    #[error("Error rendering TUI")]
    RenderError(
        #[source]
        #[from]
        thedes_tui::RenderError,
    ),
    #[error("inconsistent save name buffer reset")]
    SetSaveNameBuffer(#[source] input::InvalidNewBuffer),
    #[error("inconsistent main menu option list")]
    UnknownNewSaveMenuOption(#[source] menu::UnknownOption<NewSaveMenuOption>),
    #[error("inconsistent seed buffer size")]
    SetSeedBuffer(#[source] input::InvalidNewBuffer),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum MainMenuOption {
    New,
    Load,
    Delete,
    Settings,
    Help,
    Quit,
}

impl fmt::Display for MainMenuOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::New => write!(f, "new game"),
            Self::Load => write!(f, "load game"),
            Self::Delete => write!(f, "delete game"),
            Self::Settings => write!(f, "settings"),
            Self::Help => write!(f, "help"),
            Self::Quit => write!(f, "quit"),
        }
    }
}

impl menu::OptionItem for MainMenuOption {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum NewSaveMenuOption {
    Create,
    SetName,
    SetSeed,
}

impl fmt::Display for NewSaveMenuOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Create => write!(f, "create"),
            Self::SetSeed => write!(f, "set seed"),
            Self::SetName => write!(f, "set save name"),
        }
    }
}

impl menu::OptionItem for NewSaveMenuOption {}

#[derive(Debug, Clone)]
struct App {
    main_menu: Menu<MainMenuOption, NonCancellable>,
    new_save_name_input: InputDialog<fn(char) -> bool, Cancellable>,
    new_save_menu: Menu<NewSaveMenuOption, Cancellable>,
    seed_input: InputDialog<fn(char) -> bool, Cancellable>,
    invalid_seed_info: InfoDialog,
    invalid_save_name_info: InfoDialog,
    state: RootAppState,
}

impl App {
    fn new() -> Result<Self, AppInitError> {
        Ok(Self {
            main_menu: Menu::new(menu::Config {
                base: menu::BaseConfig::new("=== T H E D E S ==="),
                cancellability: NonCancellable,
                options: menu::Options::with_initial(MainMenuOption::New)
                    .add(MainMenuOption::Load)
                    .add(MainMenuOption::Delete)
                    .add(MainMenuOption::Settings)
                    .add(MainMenuOption::Help)
                    .add(MainMenuOption::Quit),
            }),
            new_save_name_input: InputDialog::new(input::Config {
                base: input::BaseConfig::new("NEW SAVE NAME")
                    .with_max_graphemes(32)
                    .map_err(AppInitError::MaxSaveNameSize)?,
                cancellability: Cancellable::new(),
            }),
            new_save_menu: Menu::new(menu::Config {
                base: menu::BaseConfig::new("NEW GAME"),
                cancellability: Cancellable::new(),
                options: menu::Options::with_initial(NewSaveMenuOption::Create)
                    .add(NewSaveMenuOption::SetName)
                    .add(NewSaveMenuOption::SetSeed),
            }),
            seed_input: InputDialog::new(input::Config {
                base: input::BaseConfig::new("NEW SAVE SEED")
                    .with_max_graphemes(8)
                    .map_err(AppInitError::MaxSeedSize)?,
                cancellability: Cancellable::new(),
            }),
            invalid_seed_info: InfoDialog::new(
                info::Config::new("Invalid seed!").with_message(
                    "Seeds must be composed of hexadecimal digits (0-9, a-f, \
                     A-F), having at least one digit",
                ),
            ),
            invalid_save_name_info: InfoDialog::new(
                info::Config::new("Invalid save name!")
                    .with_message("Save names cannot be empty"),
            ),
            state: RootAppState::MainMenu,
        })
    }

    fn on_tick(&mut self, tick: &mut Tick) -> Result<bool, AppExecError> {
        match &mut self.state {
            RootAppState::MainMenu => {
                if !self.main_menu.on_tick(tick)? {
                    match self.main_menu.selection() {
                        MainMenuOption::New => {
                            self.state = RootAppState::NewSaveIntro;
                            self.new_save_name_input
                                .set_buffer([])
                                .map_err(AppExecError::SetSaveNameBuffer)?;
                            self.new_save_name_input
                                .cancellability_mut()
                                .set_cancel_state(false);
                        },
                        MainMenuOption::Load => todo!(),
                        MainMenuOption::Delete => todo!(),
                        MainMenuOption::Help => todo!(),
                        MainMenuOption::Settings => todo!(),
                        MainMenuOption::Quit => return Ok(false),
                    }
                }
            },

            RootAppState::NewSaveIntro => {
                if !self.new_save_name_input.on_tick(tick)? {
                    match self.new_save_name_input.selection() {
                        Some(name) => {
                            if name.is_empty() {
                                self.state = RootAppState::InvalidNewSaveName;
                            } else {
                                let seed = thread_rng().next_u32();
                                self.state =
                                    RootAppState::NewSave(NewSaveState {
                                        name,
                                        seed,
                                        screen: NewSaveScreen::Main,
                                    });
                                self.new_save_menu
                                    .select(NewSaveMenuOption::Create)
                                    .map_err(
                                        AppExecError::UnknownNewSaveMenuOption,
                                    )?;
                                self.seed_input
                                    .set_buffer(format!("{seed:x}").chars())
                                    .map_err(AppExecError::SetSeedBuffer)?;
                            }
                        },
                        None => self.state = RootAppState::MainMenu,
                    }
                }
            },

            RootAppState::InvalidNewSaveName => {
                if !self.invalid_save_name_info.on_tick(tick)? {
                    self.state = RootAppState::NewSaveIntro;
                }
            },

            RootAppState::NewSave(new_save_state) => {
                match &mut new_save_state.screen {
                    NewSaveScreen::Main => {
                        if !self.new_save_menu.on_tick(tick)? {
                            match self.new_save_menu.selection() {
                                Some(NewSaveMenuOption::Create) => {
                                    self.state = RootAppState::Game;
                                },
                                Some(NewSaveMenuOption::SetName) => {
                                    new_save_state.screen =
                                        NewSaveScreen::SetName;
                                },
                                Some(NewSaveMenuOption::SetSeed) => {
                                    new_save_state.screen =
                                        NewSaveScreen::SetSeed;
                                },
                                None => self.state = RootAppState::MainMenu,
                            }
                        }
                    },

                    NewSaveScreen::SetName => {
                        if !self.new_save_name_input.on_tick(tick)? {
                            if let Some(name) =
                                self.new_save_name_input.selection()
                            {
                                if name.is_empty() {
                                    new_save_state.screen =
                                        NewSaveScreen::InvalidSaveName;
                                } else {
                                    new_save_state.name = name;
                                }
                            } else {
                                self.new_save_name_input
                                    .set_buffer(new_save_state.name.chars())
                                    .map_err(AppExecError::SetSaveNameBuffer)?;
                                self.seed_input
                                    .cancellability_mut()
                                    .set_cancel_state(false);
                            }
                            new_save_state.screen = NewSaveScreen::Main;
                        }
                    },

                    NewSaveScreen::SetSeed => {
                        if !self.seed_input.on_tick(tick)? {
                            new_save_state.screen = NewSaveScreen::Main;
                            if let Some(seed_str) = self.seed_input.selection()
                            {
                                if let Ok(seed) =
                                    u32::from_str_radix(&seed_str, 16)
                                {
                                    new_save_state.seed = seed;
                                } else {
                                    new_save_state.screen =
                                        NewSaveScreen::InvalidSeed;
                                }
                            } else {
                                self.seed_input
                                    .set_buffer(
                                        format!("{:x}", new_save_state.seed)
                                            .chars(),
                                    )
                                    .map_err(AppExecError::SetSeedBuffer)?;
                                self.seed_input
                                    .cancellability_mut()
                                    .set_cancel_state(false);
                            }
                        }
                    },

                    NewSaveScreen::InvalidSaveName => {
                        if !self.invalid_save_name_info.on_tick(tick)? {
                            new_save_state.screen = NewSaveScreen::SetName;
                        }
                    },

                    NewSaveScreen::InvalidSeed => {
                        if !self.invalid_seed_info.on_tick(tick)? {
                            new_save_state.screen = NewSaveScreen::SetSeed;
                        }
                    },
                }
            },

            RootAppState::Game => todo!(),
        }
        Ok(true)
    }
}

#[derive(Debug, Clone)]
enum RootAppState {
    MainMenu,
    NewSaveIntro,
    InvalidNewSaveName,
    NewSave(NewSaveState),
    Game,
}

#[derive(Debug, Clone)]
struct NewSaveState {
    name: String,
    seed: u32,
    screen: NewSaveScreen,
}

#[derive(Debug, Clone)]
enum NewSaveScreen {
    Main,
    SetName,
    SetSeed,
    InvalidSeed,
    InvalidSaveName,
}

const LOG_ENABLED_ENV_VAR: &'static str = "THEDES_LOG";
const LOG_LEVEL_ENV_VAR: &'static str = "THEDES_LOG_LEVEL";
const LOG_PATH_ENV_VAR: &'static str = "THEDES_LOG_PATH";

fn setup_logger() -> Result<(), ProgramError> {
    let mut options = fs::OpenOptions::new();

    options.write(true).append(true).create(true).truncate(false);

    let path = match env::var_os(LOG_PATH_ENV_VAR) {
        Some(path) => path.into(),
        None => {
            let now = chrono::Local::now();
            let stem = format!(
                "log_{:04}-{:02}-{:02}_{:02}-{:02}-{:02}.txt",
                now.year(),
                now.month(),
                now.day(),
                now.hour(),
                now.minute(),
                now.second(),
            );
            match directories::ProjectDirs::from(
                "io.github",
                "brunoczim",
                "Thedes",
            ) {
                Some(dirs) => dirs.cache_dir().join(stem),
                None => stem.into(),
            }
        },
    };

    if let Some(dir) = path.parent() {
        fs::create_dir_all(&dir).map_err(|cause| {
            ProgramError::OpenLogFile { path: path.clone(), cause }
        })?;
    }

    let file = options
        .open(&path)
        .map_err(|cause| ProgramError::OpenLogFile { path, cause })?;

    registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(Arc::new(file))
                .with_filter(
                    EnvFilter::builder()
                        .with_default_directive(LevelFilter::INFO.into())
                        .with_env_var(LOG_LEVEL_ENV_VAR)
                        .from_env()?,
                ),
        )
        .init();

    Ok(())
}

fn setup_panic_handler() {
    panic::set_hook(Box::new(|info| {
        thedes_tui::panic::emergency_restore();
        eprintln!("{}", info);
        tracing::error!("{}\n", info);
        let backtrace = Backtrace::capture();
        tracing::error!("backtrace:\n{}\n", backtrace);
    }));
}

fn try_main() -> Result<(), ProgramError> {
    env::set_var("RUST_BACKTRACE", "1");

    setup_panic_handler();

    let mut log_enabled = false;
    if env::var_os(LOG_LEVEL_ENV_VAR).is_some()
        || env::var_os(LOG_PATH_ENV_VAR).is_some()
    {
        log_enabled = true;
    }
    if let Some(log_enabled_var) = env::var_os(LOG_ENABLED_ENV_VAR) {
        if log_enabled_var.eq_ignore_ascii_case("true")
            || log_enabled_var.eq_ignore_ascii_case("t")
            || log_enabled_var.eq_ignore_ascii_case("on")
            || log_enabled_var.eq_ignore_ascii_case("yes")
            || log_enabled_var.eq_ignore_ascii_case("y")
            || log_enabled_var == "1"
        {
            log_enabled = true;
        } else if log_enabled_var.eq_ignore_ascii_case("false")
            || log_enabled_var.eq_ignore_ascii_case("f")
            || log_enabled_var.eq_ignore_ascii_case("off")
            || log_enabled_var.eq_ignore_ascii_case("no")
            || log_enabled_var.eq_ignore_ascii_case("n")
            || log_enabled_var == "0"
        {
            log_enabled = false;
        }
    }

    if log_enabled {
        setup_logger()?;
    }

    let mut app = App::new()?;

    thedes_tui::Config::default().run(move |tick| app.on_tick(tick))?;

    Ok(())
}

fn main() {
    if let Err(error) = try_main() {
        eprintln!("thedes found a fatal error!");
        let mut current_error = Some(&error as &dyn Error);
        while let Some(error) = current_error {
            eprintln!("caused by:");
            eprintln!("  {error}");
            current_error = error.source();
        }
    }
}
