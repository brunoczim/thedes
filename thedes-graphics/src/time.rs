use thedes_domain::time::{CircadianCycleStep, LunarPhase, Time};
use thedes_tui::{
    color::{Brightness, Color, ColorPair, LegacyRgb},
    grapheme,
    tile::Tile,
};
use thiserror::Error;

fn horizon_sun_color() -> Color {
    LegacyRgb::new(5, 4, 2).into()
}

fn horizon_color() -> Color {
    LegacyRgb::new(5, 3, 1).into()
}

fn bright_sun_color() -> Color {
    LegacyRgb::new(5, 5, 3).into()
}

fn new_moon_color() -> Color {
    LegacyRgb::new(0, 1, 1).into()
}

fn bright_moon_color() -> Color {
    LegacyRgb::new(3, 5, 5).into()
}

fn brightest_moon_color() -> Color {
    LegacyRgb::new(4, 5, 5).into()
}

pub fn light(circadian_cycle_step: CircadianCycleStep) -> Brightness {
    match circadian_cycle_step {
        CircadianCycleStep::Sunrise => {
            Brightness { level: Brightness::MAX.level / 9 * 7 }
        },
        CircadianCycleStep::DayLight => Brightness::MAX,
        CircadianCycleStep::Sunset => {
            Brightness { level: Brightness::MAX.level / 9 * 5 }
        },
        CircadianCycleStep::Night => {
            Brightness { level: Brightness::MAX.level / 9 * 3 }
        },
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to register grapheme")]
    Grapheme(
        #[from]
        #[source]
        grapheme::NotGrapheme,
    ),
}

pub fn circadian_cycle_icon(
    time: &Time,
    graphemes: &mut grapheme::Registry,
) -> Result<[Tile; 2], Error> {
    let tiles = match time.circadian_cycle_step() {
        CircadianCycleStep::Sunrise => [
            Tile {
                grapheme: graphemes.get_or_register("*")?,
                colors: ColorPair {
                    foreground: horizon_sun_color(),
                    ..Default::default()
                },
            },
            Tile {
                grapheme: graphemes.get_or_register("↑")?,
                colors: ColorPair {
                    foreground: horizon_color(),
                    ..Default::default()
                },
            },
        ],
        CircadianCycleStep::DayLight => [
            Tile {
                grapheme: graphemes.get_or_register("*")?,
                colors: ColorPair {
                    foreground: bright_sun_color(),
                    ..Default::default()
                },
            },
            Tile {
                grapheme: graphemes.get_or_register(" ")?,
                colors: Default::default(),
            },
        ],
        CircadianCycleStep::Sunset => [
            Tile {
                grapheme: graphemes.get_or_register("*")?,
                colors: ColorPair {
                    foreground: horizon_sun_color(),
                    ..Default::default()
                },
            },
            Tile {
                grapheme: graphemes.get_or_register("↓")?,
                colors: ColorPair {
                    foreground: horizon_color(),
                    ..Default::default()
                },
            },
        ],
        CircadianCycleStep::Night => match time.lunar_phase() {
            LunarPhase::New => [
                Tile {
                    grapheme: graphemes.get_or_register("●︎")?,
                    colors: ColorPair {
                        foreground: new_moon_color(),
                        ..Default::default()
                    },
                },
                Tile {
                    grapheme: graphemes.get_or_register(" ")?,
                    colors: ColorPair::default(),
                },
            ],
            LunarPhase::WaxingCrescent => [
                Tile {
                    grapheme: graphemes.get_or_register("(")?,
                    colors: ColorPair {
                        foreground: bright_moon_color(),
                        ..Default::default()
                    },
                },
                Tile {
                    grapheme: graphemes.get_or_register(" ")?,
                    colors: ColorPair::default(),
                },
            ],
            LunarPhase::FirstQuarter => [
                Tile {
                    grapheme: graphemes.get_or_register("(")?,
                    colors: ColorPair {
                        foreground: bright_moon_color(),
                        ..Default::default()
                    },
                },
                Tile {
                    grapheme: graphemes.get_or_register("▎")?,
                    colors: ColorPair {
                        foreground: bright_moon_color(),
                        ..Default::default()
                    },
                },
            ],
            LunarPhase::WaxingGibbous => [
                Tile {
                    grapheme: graphemes.get_or_register("◖")?,
                    colors: ColorPair {
                        foreground: bright_moon_color(),
                        ..Default::default()
                    },
                },
                Tile {
                    grapheme: graphemes.get_or_register(")")?,
                    colors: ColorPair {
                        foreground: bright_moon_color(),
                        ..Default::default()
                    },
                },
            ],
            LunarPhase::Full => [
                Tile {
                    grapheme: graphemes.get_or_register("●︎")?,
                    colors: ColorPair {
                        foreground: brightest_moon_color(),
                        ..Default::default()
                    },
                },
                Tile {
                    grapheme: graphemes.get_or_register(" ")?,
                    colors: ColorPair::default(),
                },
            ],
            LunarPhase::WaningGibbous => [
                Tile {
                    grapheme: graphemes.get_or_register("(")?,
                    colors: ColorPair {
                        foreground: bright_moon_color(),
                        ..Default::default()
                    },
                },
                Tile {
                    grapheme: graphemes.get_or_register("◗")?,
                    colors: ColorPair {
                        foreground: bright_moon_color(),
                        ..Default::default()
                    },
                },
            ],
            LunarPhase::LastQuarter => [
                Tile {
                    grapheme: graphemes.get_or_register("▐")?,
                    colors: ColorPair {
                        foreground: bright_moon_color(),
                        ..Default::default()
                    },
                },
                Tile {
                    grapheme: graphemes.get_or_register(")")?,
                    colors: ColorPair {
                        foreground: bright_moon_color(),
                        ..Default::default()
                    },
                },
            ],
            LunarPhase::WaningCrescent => [
                Tile {
                    grapheme: graphemes.get_or_register(")")?,
                    colors: ColorPair {
                        foreground: bright_moon_color(),
                        ..Default::default()
                    },
                },
                Tile {
                    grapheme: graphemes.get_or_register(" ")?,
                    colors: ColorPair::default(),
                },
            ],
        },
    };
    Ok(tiles)
}
