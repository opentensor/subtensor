use super::*;
use quote::ToTokens;
use syn::braced;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Colon;
use syn::visit::Visit;
use syn::{File, Ident, ItemMacro, Path, Token, Visibility};

pub struct RequireExplicitPalletIndex;

impl Lint for RequireExplicitPalletIndex {
    fn lint(source: &File) -> Result {
        let mut visitor = ConstructRuntimeVisitor::default();

        visitor.visit_file(source);

        if !visitor.errors.is_empty() {
            return Err(visitor.errors);
        }

        Ok(())
    }
}

#[derive(Default)]
struct ConstructRuntimeVisitor {
    errors: Vec<syn::Error>,
}

impl<'ast> Visit<'ast> for ConstructRuntimeVisitor {
    fn visit_item_macro(&mut self, node: &'ast ItemMacro) {
        if node.mac.path.is_ident("construct_runtime") {
            // Token stream parsing logic
            let tokens = node.mac.tokens.clone();
            let runtime_entries = syn::parse2::<ConstructRuntimeEntries>(tokens).unwrap();
            for entry in runtime_entries.entries {
                // Check if the entry is missing an explicit index
                if entry.index.is_none() {
                    self.errors.push(syn::Error::new(
                        entry.pallet_name.span(),
                        format!(
                            "Pallet `{}` does not have an explicit index in construct_runtime!",
                            entry.pallet_name.to_token_stream().to_string().trim()
                        ),
                    ));
                }
            }
        }

        // Continue visiting the rest of the file
        syn::visit::visit_item_macro(self, node);
    }
}

struct ConstructRuntimeEntries {
    entries: Punctuated<PalletEntry, Token![,]>,
}

impl Parse for ConstructRuntimeEntries {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(ConstructRuntimeEntries {
            entries: input.parse_terminated(PalletEntry::parse, Token![,])?,
        })
    }
}

struct PalletEntry {
    visibility: Option<Visibility>,
    pallet_name: Path,
    components: Option<PalletComponents>,
    index: Option<syn::LitInt>, // Now index can be None (i.e., missing)
}

impl Parse for PalletEntry {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Optionally parse visibility (e.g., `pub`)
        let visibility: Option<Visibility> = input.parse().ok();

        // Parse the pallet name (with possible generics and paths like `pallet_collective::<Instance1>::{ Pallet, Call, Storage }`)
        let pallet_name = parse_complex_pallet_path(input)?;

        // Optionally parse the index if it's present
        let index = if input.peek(Colon) {
            input.parse::<Colon>()?;
            Some(input.parse::<syn::LitInt>()?)
        } else {
            None // Missing index is allowed during parsing
        };

        Ok(PalletEntry {
            visibility,
            pallet_name,
            components: None, // Components will be handled directly in `parse_complex_pallet_path`
            index,
        })
    }
}

fn parse_complex_pallet_path(input: ParseStream) -> syn::Result<Path> {
    let mut path = Path::parse_mod_style(input)?;

    // Check if there are generics like `::<Instance1>`
    if input.peek(syn::token::Lt) {
        let _generics: syn::AngleBracketedGenericArguments = input.parse()?;
    }

    // Now check for nested components in `{ Pallet, Call, Storage }`
    if input.peek(syn::token::Brace) {
        let content;
        braced!(content in input);
        let _: Punctuated<Ident, Token![,]> = content.parse_terminated(Ident::parse, Token![,])?;
    }

    Ok(path)
}

struct PalletComponents {
    components: Punctuated<Ident, Token![,]>,
}

impl Parse for PalletComponents {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(PalletComponents {
            components: input.parse_terminated(Ident::parse, Token![,])?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lint_macro(input: &str) -> Result {
        let item_macro: ItemMacro = syn::parse_str(input).expect("should only use on a macro");
        let mut visitor = ConstructRuntimeVisitor::default();
        visitor.visit_item_macro(&item_macro);
        if !visitor.errors.is_empty() {
            return Err(visitor.errors);
        }
        Ok(())
    }

    #[test]
    fn test_no_pallet_index() {
        let input = r#"
            construct_runtime!(
                PalletA,
                PalletB
            );
        "#;
        assert!(lint_macro(input).is_err());
    }

    #[test]
    fn test_with_pallet_index() {
        let input = r#"
            construct_runtime!(
                PalletA: 0,
                PalletB: 1
            );
        "#;
        assert!(lint_macro(input).is_ok());
    }

    #[test]
    fn test_mixed_pallet_index() {
        let input = r#"
            construct_runtime!(
                PalletA,
                PalletB: 1
            );
        "#;
        assert!(lint_macro(input).is_err());
    }

    #[test]
    fn test_with_visibility_and_index() {
        let input = r#"
            construct_runtime!(
                pub PalletA: 0,
                PalletB: 1
            );
        "#;
        assert!(lint_macro(input).is_ok());
    }

    #[test]
    fn test_with_generic_and_index() {
        let input = r#"
            construct_runtime!(
                PalletA,
                pallet_collective::<Instance1>::{ Pallet, Call, Storage }: 1
            );
        "#;
        assert!(lint_macro(input).is_ok());
    }

    #[test]
    fn test_with_nested_and_missing_index() {
        let input = r#"
            construct_runtime!(
                PalletA,
                pallet_collective::<Instance1>::{ Pallet, Call, Storage }
            );
        "#;
        assert!(lint_macro(input).is_err());
    }
}
