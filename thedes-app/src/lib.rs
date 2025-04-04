use thedes_tui::core::{
    color::{BasicColor, ColorPair},
    event::{Event, Key, KeyEvent},
    geometry::CoordPair,
    mutation::{MutationExt, Set},
    screen,
    tile::{MutateColors, MutateGrapheme},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to render TUI")]
    RenderText(
        #[from]
        #[source]
        thedes_tui::text::Error,
    ),
    #[error("Failed to interact with screen canvas")]
    CanvasFlush(
        #[from]
        #[source]
        thedes_tui::core::screen::FlushError,
    ),
}

pub async fn root(mut app: thedes_tui::core::App) -> Result<(), Error> {
    let mut timer = app.timer.new_participant();

    let min = CoordPair { y: 1, x: 0 };
    let max = app.canvas.size() - 1;
    let mut curr = min + (min + max) / 2;

    'main: loop {
        app.canvas
            .queue([screen::Command::ClearScreen(BasicColor::Black.into())]);
        thedes_tui::text::inline(
            &mut app,
            CoordPair { y: 0, x: 0 },
            "Hello, World! Q/Esc to quit, HJKL to move",
            ColorPair {
                foreground: BasicColor::Black.into(),
                background: BasicColor::LightGreen.into(),
            },
        )?;
        app.canvas.queue([screen::Command::new_mutation(
            curr,
            MutateGrapheme(Set('O'.into())).then(MutateColors(Set(
                ColorPair {
                    foreground: BasicColor::White.into(),
                    background: BasicColor::Black.into(),
                },
            ))),
        )]);
        if app.canvas.flush().is_err() {
            tracing::info!("Screen command receiver disconnected");
            break;
        }

        let Ok(events) = app.events.read_until_now() else {
            tracing::info!("Event sender disconnected");
            break;
        };
        for event in events {
            let Event::Key(KeyEvent {
                alt: false,
                ctrl: false,
                shift: false,
                main_key,
            }) = event
            else {
                continue;
            };

            match main_key {
                Key::Char('q') | Key::Char('Q') | Key::Esc => break 'main,
                Key::Char('h') | Key::Char('H') => {
                    curr.x = curr.x.saturating_sub(1).max(min.x).min(max.x);
                },
                Key::Char('j') | Key::Char('J') => {
                    curr.y = curr.y.saturating_add(1).max(min.y).min(max.y);
                },
                Key::Char('k') | Key::Char('K') => {
                    curr.y = curr.y.saturating_sub(1).max(min.y).min(max.y);
                },
                Key::Char('l') | Key::Char('L') => {
                    curr.x = curr.x.saturating_add(1).max(min.x).min(max.x);
                },
                _ => (),
            }
        }

        timer.tick().await;
    }

    Ok(())
}
