use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use syn::{parse2, visit_mut::visit_item_struct_mut, Error, ItemStruct, LitStr, Result};

mod visitor;
use visitor::*;

/// Freezes the layout of a struct to the current hash of its fields, ensuring that future
/// changes require updating the hash.
///
/// If you ever get in a loop where rust analyzer insists that the hash is wrong each time you
/// update, but `cargo check` is passing, you should use [`freeze_struct_ignore_ra`] instead.
#[proc_macro_attribute]
pub fn freeze_struct(attr: TokenStream, tokens: TokenStream) -> TokenStream {
    match freeze_struct_impl(attr, tokens, false) {
        Ok(item_struct) => item_struct.to_token_stream().into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// More permissive version of [`freeze_struct`] that ignores the hash check when running in
/// rust-analyzer since in some rare situations rust-analyzer will generate an incorrect hash code
#[proc_macro_attribute]
pub fn freeze_struct_ignore_ra(attr: TokenStream, tokens: TokenStream) -> TokenStream {
    match freeze_struct_impl(attr, tokens, true) {
        Ok(item_struct) => item_struct.to_token_stream().into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn freeze_struct_impl(
    attr: impl Into<TokenStream2>,
    tokens: impl Into<TokenStream2>,
    ignore_ra: bool,
) -> Result<ItemStruct> {
    let attr = attr.into();
    let tokens = tokens.into();

    let item = parse2::<ItemStruct>(tokens)?;
    let mut item_clone = item.clone();

    let calculated_hash = generate_hash(&item);
    let calculated_hash_hex = format!("{:x}", calculated_hash);

    if attr.is_empty() {
        return Err(Error::new_spanned(item,
            format!("You must provide a hashcode in the `freeze_struct` attribute to freeze this struct.\n\n\
            expected hashcode: `#[freeze_struct(\"{calculated_hash_hex}\")]`"),
        ));
    }
    let hash_lit = parse2::<LitStr>(attr)?;
    let provided_hash_hex = hash_lit.value().to_lowercase();

    let mut visitor = CleanDocComments::new();
    visit_item_struct_mut(&mut visitor, &mut item_clone);

    if ignore_ra && is_rust_analyzer() {
        return Ok(item);
    }

    if provided_hash_hex != calculated_hash_hex {
        return Err(Error::new_spanned(item,
            format!(
                "You have made a non-trivial change to this struct and the provided hashcode no longer matches:\n{} != {}\n\n\
                If this was intentional, please update the hashcode in the `freeze_struct` attribute to:\n\
                {}\n\nNote that if you are changing a storage struct in any way, including simply re-ordering fields, \
                you will need a migration to prevent data corruption.",
                provided_hash_hex, calculated_hash_hex, calculated_hash_hex
            ),
        ));
    }
    Ok(item)
}

/// Returns true if the current build is being run by rust-analyzer.
fn is_rust_analyzer() -> bool {
    std::env::var("RUSTC_WRAPPER")
        .map(|v| v.contains("rust-analyzer"))
        .unwrap_or(false)
}
