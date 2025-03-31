use num::CheckedAdd;
pub use style::Style;
use thedes_tui_core::{
    App,
    color::ColorPair,
    geometry::{Coord, CoordPair},
    grapheme,
    mutation::{MutationExt, Set},
    screen::Command,
    tile::{MutateColors, MutateGrapheme, Tile},
};
use thiserror::Error;

mod style;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Size {0} is too big for an inline text")]
    InlineTextTooBig(usize),
    #[error("Inline text with size {size} overflows starting at {start}")]
    InlineTextOverflow { start: CoordPair, size: usize },
}

pub fn inline(
    app: &mut App,
    canvas_point: CoordPair,
    input: &str,
    colors: ColorPair,
) -> Result<Coord, Error> {
    let graphemes: Vec<_> =
        app.grapheme_registry.get_or_register_many(input).collect();
    let size = graphemes.len();
    let mut offset: Coord = 0;
    for grapheme in graphemes {
        let offset_canvas_point = canvas_point
            .checked_add(&CoordPair { y: 0, x: offset })
            .ok_or_else(|| Error::InlineTextOverflow {
                start: canvas_point,
                size,
            })?;
        app.canvas.queue([Command::new_mutation(
            offset_canvas_point,
            Set(Tile { colors, grapheme }),
        )]);
        offset = offset
            .checked_add(1)
            .ok_or_else(|| Error::InlineTextTooBig(size))?;
    }
    Ok(offset)
}

pub fn styled(
    app: &mut App,
    input: &str,
    style: &Style,
) -> Result<Coord, Error> {
    let graphemes: Vec<_> =
        app.grapheme_registry.get_or_register_many(input).collect();
    let mut slice = &graphemes[..];
    let canvas_size = app.canvas.size();
    let size = style.make_size(canvas_size);

    let mut cursor = CoordPair { x: 0, y: style.top_margin() };
    let mut is_inside = cursor.y - style.top_margin() < size.y;

    while !slice.is_empty() && is_inside {
        is_inside = cursor.y - style.top_margin() + 1 < size.y;
        let width = usize::from(size.x);
        let pos = find_break_pos(width, size, slice, is_inside)?;

        cursor.x = size.x - pos as Coord;
        cursor.x = cursor.x + style.left_margin();
        cursor.x = cursor.x * style.align_numer() / style.align_denom();

        let (low, high) = slice.split_at(pos);
        slice = high;

        print_slice(app, low, &style, &mut cursor)?;

        if pos != slice.len() && !is_inside {
            let elipsis = grapheme::Id::from('…');
            let mutation = MutateGrapheme(Set(elipsis))
                .then(MutateColors(*style.colors()));
            app.canvas.queue([Command::new_mutation(cursor, mutation)]);
        }

        cursor.y += 1;
    }

    Ok(cursor.y)
}

fn find_break_pos(
    width: usize,
    box_size: CoordPair,
    graphemes: &[grapheme::Id],
    is_inside: bool,
) -> Result<usize, Error> {
    let space = grapheme::Id::from(' ');
    if width <= graphemes.len() {
        let mut pos = graphemes[.. usize::from(box_size.x)]
            .iter()
            .rposition(|grapheme| *grapheme == space)
            .unwrap_or(width);
        if !is_inside {
            pos -= 1;
        }
        Ok(pos)
    } else {
        Ok(graphemes.len())
    }
}

fn print_slice(
    app: &mut App,
    slice: &[grapheme::Id],
    style: &Style,
    cursor: &mut CoordPair,
) -> Result<(), Error> {
    for grapheme in slice {
        let mutation =
            MutateGrapheme(Set(*grapheme)).then(MutateColors(*style.colors()));
        app.canvas.queue([Command::new_mutation(*cursor, mutation)]);
        cursor.x += 1;
    }

    Ok(())
}
