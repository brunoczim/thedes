/// Creates a `GString` from string literal. Panicks if the string is invalid.
#[macro_export]
macro_rules! gstring {
    [] => {
        $crate::graphics::GString::default()
    };

    [$s:expr] => {
        $crate::graphics::GString::new_lossy($s)
    };
}

/// Creates a `ColoredGString` from a list of `GString`s and colors. Panicks if
/// the string is invalid.
#[macro_export]
macro_rules! colored_gstring {
    [$($contents:expr, $color:expr);*] => {
        $crate::graphics::ColoredGString::new(
            [Some($($contents, $color),*)]
                .iter_mut()
                .map(|opt| opt.take().expect("Cannot be called twice"))
        )
    };

    [$($contents:expr, $color:expr;)*] => {
        colored_gstring![$($contents, $color);*]
    };
}

/// Creates a `GString` from various other `GString`-like fragments by
/// concatenation.
#[macro_export]
macro_rules! gconcat {
    [$($elem:expr,)*]  => {{
        (&[$($crate::graphics::string::StringOrGraphm::from(&$elem),)*])
            .iter()
            .map(|&x| x)
            .collect::<$crate::graphics::string::GString>()
    }};
    [$($elem:expr),+]  => {
        gconcat![$($elem,)*]
    };
}

/// Creates a `ColoredGString` from various other `ColoredGString`-like
/// fragments by concatenation.
#[macro_export]
macro_rules! colored_gconcat {
    [$($elem:expr,)*]  => {{
        (&[$($crate::graphics::string::StringOrTile::from(&$elem),)*])
            .iter()
            .map(|&x| x)
            .collect::<$crate::graphics::string::ColoredGString>()
    }};
    [$($elem:expr),+]  => {
        colored_gconcat![$($elem,)*]
    };
}
