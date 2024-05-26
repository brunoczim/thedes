use std::array;

pub use array::from_fn;

pub fn zip<F, T, U, const N: usize>(lhs: [T; N], rhs: [U; N]) -> [(T, U); N] {
    zip_with(lhs, rhs, |a, b| (a, b))
}

pub fn zip_with<F, T, U, V, const N: usize>(
    lhs: [T; N],
    rhs: [U; N],
    mut zipper: F,
) -> [V; N]
where
    F: FnMut(T, U) -> V,
{
    let mut iter = lhs.into_iter().zip(rhs);
    from_fn(|i| {
        let (a, b) = iter
            .next()
            .unwrap_or_else(|| panic!("inconsistent zip at index {i}"));
        zipper(a, b)
    })
}
