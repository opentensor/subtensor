use proc_macro2::TokenTree;
use syn::{Attribute, File, Meta, MetaList, Path};

pub type Result = core::result::Result<(), Vec<syn::Error>>;

/// A trait that defines custom lints that can be run within our workspace.
///
/// Each lint is run in parallel on all Rust source files in the workspace. Within a lint you
/// can issue an error the same way you would in a proc macro, and otherwise return `Ok(())` if
/// there are no errors.
pub trait Lint: Send + Sync {
    /// Lints the given Rust source file, returning a compile error if any issues are found.
    fn lint(source: &File) -> Result;
}

pub fn is_allowed(attibutes: &[Attribute]) -> bool {
    attibutes.iter().any(|attribute| {
        let Attribute {
            meta:
                Meta::List(MetaList {
                    path: Path { segments: attr, .. },
                    tokens: attr_args,
                    ..
                }),
            ..
        } = attribute
        else {
            return false;
        };

        attr.len() == 1
            && attr[0].ident == "allow"
            && attr_args.clone().into_iter().any(|arg| {
                let TokenTree::Ident(ref id) = arg else {
                    return false;
                };
                id == "unknown_lints"
            })
    })
}
