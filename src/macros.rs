/// Creates a grapheme from string literal. Panicks if the string is invalid.
#[macro_export]
macro_rules! graphemes {
    [] => {
        vec![]
    };

    [$s:expr] => {
        $crate::graphics::Grapheme::expect_iter($s).collect::<Vec<_>>()
    };
}
