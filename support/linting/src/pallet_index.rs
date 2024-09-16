use super::*;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Colon;
use syn::visit::Visit;
use syn::{File, ItemMacro, Token};

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
            let tokens = node.mac.tokens.clone();
            if let Ok(runtime_entries) = syn::parse2::<ConstructRuntimeEntries>(tokens) {
                for entry in runtime_entries.entries {
                    if entry.index.is_none() {
                        self.errors.push(syn::Error::new(
                            entry.pallet_name.span(),
                            format!(
                                "Pallet `{}` does not have an explicit index in construct_runtime!",
                                entry.pallet_name
                            ),
                        ));
                    }
                }
            }
        }

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
    pallet_name: syn::Ident,
    index: Option<syn::LitInt>,
}

impl Parse for PalletEntry {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let pallet_name: syn::Ident = input.parse()?;
        let index = if input.peek(Colon) {
            input.parse::<Colon>()?;
            Some(input.parse::<syn::LitInt>()?)
        } else {
            None
        };
        Ok(PalletEntry { pallet_name, index })
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
}
