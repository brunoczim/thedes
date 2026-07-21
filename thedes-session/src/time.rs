use thedes_domain::time::{CircadianCycleStep, LunarPhase, Time};
use thedes_tui::core::{
    color::{Brightness, Color, ColorPair, LegacyLevel, LegacyRgb},
    grapheme,
    tile::Tile,
};
use thiserror::Error;

fn horizon_sun_color() -> Color {
    LegacyRgb::new_saturating(5, 4, 2).into()
}

fn horizon_color() -> Color {
    LegacyRgb::new_saturating(5, 3, 1).into()
}

fn bright_sun_color() -> Color {
    LegacyRgb::new_saturating(5, 5, 3).into()
}

fn new_moon_color() -> Color {
    LegacyRgb::new_saturating(0, 1, 3).into()
}

fn bright_moon_color() -> Color {
    LegacyRgb::new_saturating(3, 5, 5).into()
}

fn brightest_moon_color() -> Color {
    LegacyRgb::new_saturating(4, 5, 5).into()
}

pub fn light(circadian_cycle_step: CircadianCycleStep) -> Brightness {
    match circadian_cycle_step {
        CircadianCycleStep::Sunrise => {
            Brightness::new(Brightness::MAX.level() / 9 * 7)
        },
        CircadianCycleStep::DayLight => Brightness::MAX,
        CircadianCycleStep::Sunset => {
            Brightness::new(Brightness::MAX.level() / 9 * 5)
        },
        CircadianCycleStep::Night => {
            Brightness::new(Brightness::MAX.level() / 9 * 3)
        },
    }
}

pub fn circadian_cycle_icon(
    time: Time,
    graphemes: &mut grapheme::Registry,
) -> [Tile; 2] {
    match time.circadian_cycle_step() {
        CircadianCycleStep::Sunrise => [
            Tile {
                grapheme: grapheme::Id::from('*'),
                colors: ColorPair {
                    foreground: horizon_sun_color(),
                    ..Default::default()
                },
            },
            Tile {
                grapheme: grapheme::Id::from('↑'),
                colors: ColorPair {
                    foreground: horizon_color(),
                    ..Default::default()
                },
            },
        ],
        CircadianCycleStep::DayLight => [
            Tile {
                grapheme: grapheme::Id::from('*'),
                colors: ColorPair {
                    foreground: bright_sun_color(),
                    ..Default::default()
                },
            },
            Tile {
                grapheme: grapheme::Id::from(' '),
                colors: Default::default(),
            },
        ],
        CircadianCycleStep::Sunset => [
            Tile {
                grapheme: grapheme::Id::from('*'),
                colors: ColorPair {
                    foreground: horizon_sun_color(),
                    ..Default::default()
                },
            },
            Tile {
                grapheme: grapheme::Id::from('↓'),
                colors: ColorPair {
                    foreground: horizon_color(),
                    ..Default::default()
                },
            },
        ],
        CircadianCycleStep::Night => match time.lunar_phase() {
            LunarPhase::New => [
                Tile {
                    grapheme: grapheme::Id::from('○'),
                    colors: ColorPair {
                        foreground: new_moon_color(),
                        ..Default::default()
                    },
                },
                Tile {
                    grapheme: grapheme::Id::from(' '),
                    colors: ColorPair::default(),
                },
            ],
            LunarPhase::WaxingCrescent => [
                Tile {
                    grapheme: grapheme::Id::from('('),
                    colors: ColorPair {
                        foreground: bright_moon_color(),
                        ..Default::default()
                    },
                },
                Tile {
                    grapheme: grapheme::Id::from(' '),
                    colors: ColorPair::default(),
                },
            ],
            LunarPhase::FirstQuarter => [
                Tile {
                    grapheme: grapheme::Id::from('('),
                    colors: ColorPair {
                        foreground: bright_moon_color(),
                        ..Default::default()
                    },
                },
                Tile {
                    grapheme: grapheme::Id::from('▎'),
                    colors: ColorPair {
                        foreground: bright_moon_color(),
                        ..Default::default()
                    },
                },
            ],
            LunarPhase::WaxingGibbous => [
                Tile {
                    grapheme: grapheme::Id::from('◖'),
                    colors: ColorPair {
                        foreground: bright_moon_color(),
                        ..Default::default()
                    },
                },
                Tile {
                    grapheme: grapheme::Id::from(')'),
                    colors: ColorPair {
                        foreground: bright_moon_color(),
                        ..Default::default()
                    },
                },
            ],
            LunarPhase::Full => [
                Tile {
                    grapheme: grapheme::Id::from('⬤'),
                    colors: ColorPair {
                        foreground: brightest_moon_color(),
                        ..Default::default()
                    },
                },
                Tile {
                    grapheme: grapheme::Id::from(' '),
                    colors: ColorPair::default(),
                },
            ],
            LunarPhase::WaningGibbous => [
                Tile {
                    grapheme: grapheme::Id::from('('),
                    colors: ColorPair {
                        foreground: bright_moon_color(),
                        ..Default::default()
                    },
                },
                Tile {
                    grapheme: grapheme::Id::from('◗'),
                    colors: ColorPair {
                        foreground: bright_moon_color(),
                        ..Default::default()
                    },
                },
            ],
            LunarPhase::LastQuarter => [
                Tile {
                    grapheme: grapheme::Id::from('▐'),
                    colors: ColorPair {
                        foreground: bright_moon_color(),
                        ..Default::default()
                    },
                },
                Tile {
                    grapheme: grapheme::Id::from(')'),
                    colors: ColorPair {
                        foreground: bright_moon_color(),
                        ..Default::default()
                    },
                },
            ],
            LunarPhase::WaningCrescent => [
                Tile {
                    grapheme: grapheme::Id::from(')'),
                    colors: ColorPair {
                        foreground: bright_moon_color(),
                        ..Default::default()
                    },
                },
                Tile {
                    grapheme: grapheme::Id::from(' '),
                    colors: ColorPair::default(),
                },
            ],
        },
    }
}
