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

/// Measures time of a given action.
#[macro_export]
macro_rules! measure_time {
    ($expr:expr) => {{
        let then = ::std::time::Instant::now();
        let val = $expr;
        let elapsed = then.elapsed().as_nanos();
        (val, elapsed)
    }};
    ($expr:expr, $($arg:tt)+) => {{
        let (val, elapsed) = measure_time!($expr);
        ::tracing::debug!({ ?elapsed }, $($arg)+);
        val
    }};
}
