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
