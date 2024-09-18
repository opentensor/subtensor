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
            println!("Parsing construct_runtime! tokens: {}", tokens.to_string());

            let result = syn::parse2::<ConstructRuntimeEntries>(tokens);
            if let Ok(runtime_entries) = result {
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
            } else {
                // Log error
                println!("Failed to parse construct_runtime! block: {:?}", result);
                self.errors.push(result.unwrap_err());
            }
        }

        // Continue visiting the rest of the file
        syn::visit::visit_item_macro(self, node);
    }
}

#[derive(Debug)]
struct ConstructRuntimeEntries {
    entries: Punctuated<PalletEntry, Token![,]>,
}

impl Parse for ConstructRuntimeEntries {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let entries = input.parse_terminated(PalletEntry::parse, Token![,])?;
        Ok(ConstructRuntimeEntries { entries })
    }
}

#[derive(Debug)]
struct PalletEntry {
    visibility: Option<Visibility>,
    pallet_name: Path,
    components: Option<PalletComponents>,
    index: Option<syn::LitInt>,
}

impl Parse for PalletEntry {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Optionally parse visibility (e.g., `pub`)
        let visibility: Option<Visibility> = input.parse().ok();

        // Parse the pallet name (handling complex paths with generics and nested components)
        let pallet_name = parse_complex_pallet_path(input)?;

        // Optionally parse the components in `{ Pallet, Call, Storage }`
        let components = if input.peek(syn::token::Brace) {
            let content;
            braced!(content in input);
            Some(content.parse::<PalletComponents>()?)
        } else {
            None
        };

        // Optionally parse the index if it's present
        let index = if input.peek(Colon) {
            input.parse::<Colon>()?;
            Some(input.parse::<syn::LitInt>()?)
        } else {
            None
        };

        Ok(PalletEntry {
            visibility,
            pallet_name,
            components,
            index,
        })
    }
}

fn parse_complex_pallet_path(input: ParseStream) -> syn::Result<Path> {
    // Parse the base path (e.g., `pallet_collective`)
    let path = input.parse::<Path>()?;

    // If there are generics like `::<Instance1>`, handle them
    if input.peek(syn::token::Lt) {
        let _generics: syn::AngleBracketedGenericArguments = input.parse()?;
    }

    // Now handle nested components like `{ Pallet, Call, Storage }`
    if input.peek(syn::token::Brace) {
        let content;
        braced!(content in input);
        let components: Punctuated<Ident, Token![,]> =
            content.parse_terminated(Ident::parse, Token![,])?;
        println!("Parsed components: {:?}", components);
    }

    Ok(path)
}

#[derive(Debug)]
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
        lint_macro(input).unwrap_err();
    }

    #[test]
    fn test_with_pallet_index() {
        let input = r#"
            construct_runtime!(
                PalletA: 0,
                PalletB: 1
            );
        "#;
        lint_macro(input).unwrap();
    }

    #[test]
    fn test_mixed_pallet_index() {
        let input = r#"
            construct_runtime!(
                PalletA,
                PalletB: 1
            );
        "#;
        lint_macro(input).unwrap_err();
    }

    #[test]
    fn test_with_visibility_and_index() {
        let input = r#"
            construct_runtime!(
                pub PalletA: 0,
                PalletB: 1
            );
        "#;
        lint_macro(input).unwrap();
    }

    #[test]
    fn test_with_generic_and_index() {
        let input = r#"
            construct_runtime!(
                PalletA,
                pallet_collective::<Instance1>::{ Pallet, Call, Storage }: 1
            );
        "#;
        lint_macro(input).unwrap();
    }

    #[test]
    fn test_with_nested_and_missing_index() {
        let input = r#"
            construct_runtime!(
                PalletA,
                pallet_collective::<Instance1>::{ Pallet, Call, Storage }
            );
        "#;
        lint_macro(input).unwrap_err();
    }

    #[test]
    fn test_complex_construct_runtime_enum_should_fail() {
        // This test should fail because there are no explicit indices for the pallets
        let input = r#"
		construct_runtime! {
			pub enum Test {
				System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
				Balances: pallet_balances::{Pallet, Call, Config<T>, Storage, Event<T>},
				Triumvirate: pallet_collective::<Instance1>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>},
				TriumvirateMembers: pallet_membership::<Instance1>::{Pallet, Call, Storage, Event<T>, Config<T>},
				Senate: pallet_collective::<Instance2>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>},
				SenateMembers: pallet_membership::<Instance2>::{Pallet, Call, Storage, Event<T>, Config<T>},
				SubtensorModule: pallet_subtensor::{Pallet, Call, Storage, Event<T>},
				Utility: pallet_utility::{Pallet, Call, Storage, Event},
				Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>},
				Preimage: pallet_preimage::{Pallet, Call, Storage, Event<T>},
			}
		}
        "#;

        // This should fail because there are no explicit indices
        lint_macro(input).unwrap_err();
    }

    #[test]
    fn test_complex_construct_runtime_struct() {
        let input = r#"
		construct_runtime! {
			pub struct Runtime { 
				System : frame_system = 0, 
				RandomnessCollectiveFlip : pallet_insecure_randomness_collective_flip = 1, 
				Timestamp : pallet_timestamp = 2, 
				Aura : pallet_aura = 3, 
				Grandpa : pallet_grandpa = 4, 
				Balances : pallet_balances = 5, 
				TransactionPayment : pallet_transaction_payment = 6, 
				SubtensorModule : pallet_subtensor = 7, 
				Triumvirate : pallet_collective::<Instance1>::{ Pallet, Call, Storage, Origin<T>, Event<T>, Config<T> } = 8, 
				TriumvirateMembers : pallet_membership::<Instance1>::{ Pallet, Call, Storage, Event<T>, Config<T> } = 9, 
				SenateMembers : pallet_membership::<Instance2>::{ Pallet, Call, Storage, Event<T>, Config<T> } = 10, 
				Utility : pallet_utility = 11, 
				Sudo : pallet_sudo = 12, 
				Multisig : pallet_multisig = 13, 
				Preimage : pallet_preimage = 14, 
				Scheduler : pallet_scheduler = 15, 
				Proxy : pallet_proxy = 16, 
				Registry : pallet_registry = 17, 
				Commitments : pallet_commitments = 18, 
				AdminUtils : pallet_admin_utils = 19, 
				SafeMode : pallet_safe_mode = 20 
			}
		}
        "#;

        lint_macro(input).unwrap();
    }
}
