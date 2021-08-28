use crate::{
    error::Result,
    storage::{ensure_dir, paths},
};
use andiskaz::{
    string::TermString,
    terminal::Terminal,
    tstring,
    ui::menu::{Menu, MenuOption},
};
use std::{io::ErrorKind::NotFound, path::PathBuf};
use tokio::fs;
use toml::{de, ser};

/// All settings.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct Settings {
    /// Whether debug mode is enabled.
    pub debug: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self { debug: false }
    }
}

impl Settings {
    /// Saves the settings in a file.
    pub async fn save(&self) -> Result<()> {
        let path = path()?;
        ensure_dir(path.parent().expect("not root")).await?;

        let string = ser::to_string(self)?;
        fs::write(path, string.as_bytes()).await?;
        Ok(())
    }

    /// Applies the given option to these settings.
    pub fn apply_option(&mut self, opt: &SettingsOption) {
        match opt {
            SettingsOption::Done => (),
            SettingsOption::Debug(val) => self.debug = *val,
        }
    }

    /// Applies the given options to these settings.
    pub fn apply_options(&mut self, opts: &[SettingsOption]) {
        for opt in opts {
            self.apply_option(opt)
        }
    }
}

/// Path to the settings file.
pub fn path() -> Result<PathBuf> {
    let dirs = paths()?;
    let mut path = PathBuf::from(dirs.config_dir());
    path.push("settings.toml");
    Ok(path)
}

/// Opens the settings file and returns its decoded contents.
pub async fn open() -> Result<Settings> {
    let path = path()?;
    ensure_dir(path.parent().expect("not root")).await?;

    let settings = match fs::read_to_string(path).await {
        Ok(contents) => de::from_str(&contents)?,
        Err(err) => {
            if err.kind() == NotFound {
                Settings::default()
            } else {
                Err(err)?
            }
        },
    };

    Ok(settings)
}

/// Menu shown when settings are required.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsOption {
    /// Selected when the player is done with the settings.
    Done,
    /// Selected to toggle debug.
    Debug(bool),
}

impl SettingsOption {
    /// Creates a new menu of settings options.
    pub fn menu(settings: &Settings) -> Menu<Self> {
        Menu::new(
            tstring!["Ɔ==C Settings Ɔ==C"],
            vec![SettingsOption::Done, SettingsOption::Debug(settings.debug)],
        )
    }

    /// Executes the given option. Returns whether the program should still show
    /// this menu.
    pub async fn exec(&mut self, _term: &Terminal) -> Result<bool> {
        match self {
            SettingsOption::Done => Ok(false),
            SettingsOption::Debug(val) => {
                *val = !*val;
                Ok(true)
            },
        }
    }
}

impl MenuOption for SettingsOption {
    fn name(&self) -> TermString {
        match self {
            SettingsOption::Done => tstring!["Done."],
            SettingsOption::Debug(debug) => tstring!({
                let mut buf = String::from("[toggle] debug = ");
                buf.push_str(if *debug { "On" } else { "Off" });
                buf
            }),
        }
    }
}
