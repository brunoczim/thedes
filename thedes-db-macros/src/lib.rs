use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use thedes_db_core::rid;

#[proc_macro]
pub fn rid(input: TokenStream) -> TokenStream {
    let input = proc_macro2::TokenStream::from(input);
    let data = input
        .to_string()
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>();
    let rid_bits = match rid::raw::encode(data.as_bytes()) {
        Ok(bits) => bits,
        Err(error) => {
            return syn::Error::new(input.span(), error.to_string())
                .into_compile_error()
                .into();
        },
    };
    let tokens = quote! {
        thedes_db::rid::Rid::unstable_from_raw(#rid_bits)
    };
    tokens.into()
}
