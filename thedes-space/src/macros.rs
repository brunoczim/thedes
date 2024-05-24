#[macro_export]
macro_rules! point {
    [$($coords:expr),*] => {
        $crate::Point { coords: [$($coords),*] }
    };

    [$($coords:expr,)*] => {
        $crate::point![$($coords,)*]
    }
}

#[macro_export]
macro_rules! vector {
    [$($coords:expr),*] => {
        $crate::Vector { coords: [$($coords),*] }
    };

    [$($coords:expr,)*] => {
        $crate::vector![$($coords,)*]
    }
}
