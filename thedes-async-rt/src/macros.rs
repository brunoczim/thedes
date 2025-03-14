#[macro_export]
macro_rules! dynamically_scoped {
    { $($vis:vis static $ident:ident: $ty:ty = $init:expr;)*  } => {
        thread_local! {
            $(
                $vis static $ident: $crate::local::DynScoped<$ty> =
                    $crate::local::DynScoped::new($init);
            )*
        }
    };
}
