use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse::Nothing, parse2, visit_mut::visit_item_struct_mut, ItemStruct, LitStr, Result};

mod visitor;
use visitor::*;

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

    let mut item = parse2::<ItemStruct>(tokens)?;
    let hash_lit = parse2::<LitStr>(attr)?;

    let mut visitor = CleanDocComments::new();
    visit_item_struct_mut(&mut visitor, &mut item);

    let calculated_hash = generate_hash(&item);
    let calculated_hash_hex = format!("{:x}", calculated_hash);

    todo!()
}
