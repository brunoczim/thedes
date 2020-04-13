/// Creates a `GString` from string literal. Panicks if the string is invalid.
#[macro_export]
macro_rules! gstring {
    [] => {
        $crate::graphics::GString::default()
    };

    [$s:expr] => {
        $crate::error::ResultExt::expect_display(
            $crate::graphics::GString::new($s),
            "Invalid GString"
        )
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
