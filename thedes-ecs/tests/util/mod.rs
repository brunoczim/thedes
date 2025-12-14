pub fn float_diff(left: f64, right: f64) -> f64 {
    let max_side = left.max(right);
    (left - right).abs() / max_side
}

macro_rules! assert_float_eq {
    ($left:expr, $right:expr $(, $fmt:literal $($arg:tt)*)?) => {{
        let left = $left;
        let right = $right;
        let diff = $crate::util::float_diff(f64::from(left), f64::from(right));
        assert!(
            f64::from(diff) <= f64::EPSILON,
            "Float not equal (diff: {diff}, left: {left}, right: {right})\n{}",
            format_args!(concat!("", $($fmt)?), $($arg)*),
        )
    }};
}

#[expect(unused)]
macro_rules! assert_float_ne {
    ($left:expr, $right:expr $(, $fmt:literal $($arg:tt)*)?) => {{
        let left = $left;
        let right = $right;
        let diff = $crate::util::float_diff(f64::from(left), f64::from(right));
        assert!(
            f64::from(diff) > f64::EPSILON,
            "Float is equal (diff: {diff}, left: {left}, right: {right})\n{}",
            format_args!(concat!("", $($fmt)?), $($arg)*),
        )
    }};
}
