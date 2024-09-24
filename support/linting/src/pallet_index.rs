use super::*;
use proc_macro2::TokenStream as TokenStream2;
use procedural_fork::exports::construct_runtime::parse::RuntimeDeclaration;
use quote::ToTokens;
use syn::{visit::Visit, File};

pub struct RequireExplicitPalletIndex;

impl Lint for RequireExplicitPalletIndex {
    fn lint(source: &File) -> Result {
        let mut visitor = ConstructRuntimeVisitor::new(source.to_token_stream());
        visitor.visit_file(source);

        if !visitor.errors.is_empty() {
            return Err(visitor.errors);
        }

        Ok(())
    }
}

struct ConstructRuntimeVisitor {
    original_tokens: String,
    errors: Vec<syn::Error>,
}

impl<'ast> syn::visit::Visit<'ast> for ConstructRuntimeVisitor {
    fn visit_item_macro(&mut self, node: &'ast syn::ItemMacro) {
        let is_construct_runtime = node
            .mac
            .path
            .segments
            .last()
            .map_or(false, |segment| segment.ident == "construct_runtime");

        if is_construct_runtime {
            let tokens = node.mac.tokens.clone();

            match syn::parse2::<RuntimeDeclaration>(tokens) {
                Ok(runtime_decl) => match runtime_decl {
                    RuntimeDeclaration::Explicit(runtime) => {
                        self.check_pallets_for_index(&runtime.pallets);
                    }
                    RuntimeDeclaration::ExplicitExpanded(runtime) => {
                        self.check_pallets_for_index(&runtime.pallets);
                    }
                    RuntimeDeclaration::Implicit(runtime) => {
                        // Only implicit runtime allows `None` for index
                        for pallet in runtime.pallets {
                            if pallet.index.is_none() {
                                self.errors.push(syn::Error::new(
                                    pallet.name.span(),
                                    format!(
                                        "Pallet `{}` does not have an explicit index in the implicit construct_runtime!",
                                        pallet.name.to_token_stream()
                                    ),
                                ));
                            }
                        }
                    }
                },
                Err(e) => self.errors.push(e),
            }
        }

        syn::visit::visit_item_macro(self, node);
    }
}

impl ConstructRuntimeVisitor {
    fn new(original_tokens: impl Into<TokenStream2>) -> Self {
        ConstructRuntimeVisitor {
            original_tokens: {
                let mut st = original_tokens.into().to_string();
                st.retain(|c| !c.is_whitespace());
                st
            },
            errors: Vec::new(),
        }
    }

    fn check_pallets_for_index(
        &mut self,
        pallets: &[procedural_fork::exports::construct_runtime::parse::Pallet],
    ) {
        for pallet in pallets {
            // Check for explicit index and detect missing indices
            if !self
                .original_tokens
                .contains(format!("={},", pallet.index).as_str())
            {
                // ^ HACK: FRAME's parsing code does not allow us to differentiate between an
                // automatically generated index and an explicitly provided index so we fall
                // back to the original source code here. e.g. if index is 1, we will search
                // for " = 1" in the original source code to determine if it was explicitly provided.
                self.errors.push(syn::Error::new(
                    pallet.name.span(),
                    format!(
                        "Pallet `{}` does not have an explicit index in construct_runtime!",
                        pallet.name.to_token_stream()
                    ),
                ));
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use quote::quote;

    fn lint_macro(input: proc_macro2::TokenStream) -> Result {
        let item_macro: syn::ItemMacro = syn::parse2(input).unwrap();
        let mut visitor = ConstructRuntimeVisitor::new(item_macro.to_token_stream());
        visitor.visit_item_macro(&item_macro);
        if !visitor.errors.is_empty() {
            return Err(visitor.errors);
        }
        Ok(())
    }

    #[test]
    fn test_no_pallet_index() {
        let input = quote! {
            construct_runtime! {
                pub enum Test where
                    Block = Block,
                    NodeBlock = Block,
                    UncheckedExtrinsic = UncheckedExtrinsic
                {
                    PalletA,
                    PalletB
                }
            }
        };
        lint_macro(input).unwrap_err();
    }

    #[test]
    fn test_mixed_pallet_index() {
        let input = quote! {
            construct_runtime! {
                pub enum Test where
                    Block = Block,
                    NodeBlock = Block,
                    UncheckedExtrinsic = UncheckedExtrinsic
                {
                    PalletA,
                    PalletB: 1
                }
            }
        };
        lint_macro(input).unwrap_err();
    }

    #[test]
    fn test_complex_construct_runtime_struct() {
        let input = quote! {
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
        };

        lint_macro(input).unwrap();
    }

    #[test]
    fn test_complex_construct_runtime_enum_should_fail() {
        let input = quote! {
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
        };

        lint_macro(input).unwrap_err();
    }

    #[test]
    fn test_implicit_construct_runtime_should_fail() {
        let input = quote! {
        construct_runtime! {
            pub struct Runtime {
                System: frame_system = 0,
                RandomnessCollectiveFlip: pallet_insecure_randomness_collective_flip = 1,
                Timestamp: pallet_timestamp,
                Aura: pallet_aura,
                Grandpa: pallet_grandpa,
                Balances: pallet_balances,
                TransactionPayment: pallet_transaction_payment
            }
        }
        };

        lint_macro(input).unwrap_err();
    }

    #[test]
    fn test_explicit_expanded_runtime_with_correct_index_should_pass() {
        let input = quote! {
        construct_runtime! {
            pub struct Runtime {
                System : frame_system = 0,
                Balances : pallet_balances = 1,
                ExpandedPallet: pallet_collective::{ Pallet, Call, Config<T>, Storage, Event<T> } = 2
            }
        }
        };

        lint_macro(input).unwrap();
    }

    #[test]
    fn test_explicit_expanded_runtime_with_missing_index_should_fail() {
        let input = quote! {
        construct_runtime! {
            pub struct Runtime {
                System : frame_system = 0,
                Balances : pallet_balances = 1,
                ExpandedPallet: pallet_collective::{ Pallet, Call, Config<T>, Storage, Event<T> },
                FaultyPallet: pallet_sudo
            }
        }
        };

        lint_macro(input).unwrap_err();
    }

    #[test]
    fn test_fully_qualified_construct_runtime_should_pass() {
        let input = quote! {
        frame_support::construct_runtime! {
            pub enum Test {
                System: frame_system = 1,
                Balances: pallet_balances = 2,
                AdminUtils: pallet_admin_utils = 3,
                SubtensorModule: pallet_subtensor::{Pallet, Call, Storage, Event<T>, Error<T>} = 4,
                Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>} = 5,
            }
        }
        };

        lint_macro(input).unwrap();
    }

    #[test]
    fn test_mixed_pallets_should_fail() {
        let input = quote! {
        frame_support::construct_runtime! {
            pub enum Test {
                System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>} = 1,
                Balances: pallet_balances::{Pallet, Call, Config<T>, Storage, Event<T>},
                Triumvirate: pallet_collective::<Instance1>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>} = 3,
                TriumvirateMembers: pallet_membership::<Instance1>::{Pallet, Call, Storage, Event<T>, Config<T>} = 4,
                Senate: pallet_collective::<Instance2>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>} = 5,
                SenateMembers: pallet_membership::<Instance2>::{Pallet, Call, Storage, Event<T>, Config<T>} = 6,
                SubtensorModule: pallet_subtensor::{Pallet, Call, Storage, Event<T>} = 7,
                Utility: pallet_utility::{Pallet, Call, Storage, Event} = 8,
                Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>} = 9,
                Preimage: pallet_preimage::{Pallet, Call, Storage, Event<T>} = 10,
            }
        }
        };

        lint_macro(input).unwrap_err();
    }
}
