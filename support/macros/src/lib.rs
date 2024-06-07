use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse::Nothing, parse2, ItemStruct, Result};

#[proc_macro_attribute]
pub fn freeze_struct(attr: TokenStream, tokens: TokenStream) -> TokenStream {
    match freeze_struct_impl(attr, tokens) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn freeze_struct_impl(
    attr: impl Into<TokenStream2>,
    tokens: impl Into<TokenStream2>,
) -> Result<TokenStream2> {
    let attr = attr.into();
    let tokens = tokens.into();

    let item = parse2::<ItemStruct>(tokens)?;

    todo!()
}
