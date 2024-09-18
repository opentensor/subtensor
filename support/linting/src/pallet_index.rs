use super::*;
use procedural_fork::exports::construct_runtime::parse::RuntimeDeclaration;
use quote::ToTokens;
use syn::{visit::Visit, File};

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

impl<'ast> syn::visit::Visit<'ast> for ConstructRuntimeVisitor {
    fn visit_item_macro(&mut self, node: &'ast syn::ItemMacro) {
        if node.mac.path.is_ident("construct_runtime") {
            let tokens = node.mac.tokens.clone();

            // Attempt to parse the construct_runtime invocation.
            match syn::parse2::<RuntimeDeclaration>(tokens) {
                Ok(runtime_decl) => {
                    if let RuntimeDeclaration::Explicit(runtime) = runtime_decl {
                        for pallet in runtime.pallets {
                            if pallet.index == 0 {
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
                Err(e) => self.errors.push(e),
            }
        }

        syn::visit::visit_item_macro(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    fn lint_macro(tokens: proc_macro2::TokenStream) -> Result {
        let item_macro: syn::ItemMacro = syn::parse2(tokens).expect("should only use on a macro");
        let mut visitor = ConstructRuntimeVisitor::default();
        visitor.visit_item_macro(&item_macro);
        if !visitor.errors.is_empty() {
            return Err(visitor.errors);
        }
        Ok(())
    }

    #[test]
    fn test_no_pallet_index() {
        let tokens = quote! {
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
        lint_macro(tokens).unwrap_err();
    }

    #[test]
    fn test_mixed_pallet_index() {
        let tokens = quote! {
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
        lint_macro(tokens).unwrap_err();
    }

    #[test]
    fn test_complex_construct_runtime_struct() {
        let tokens = quote! {
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
        lint_macro(tokens).unwrap();
    }

    #[test]
    fn test_complex_construct_runtime_enum_should_fail() {
        let tokens = quote! {
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
        lint_macro(tokens).unwrap_err();
    }

    #[test]
    fn test_with_multiple_instances() {
        let tokens = quote! {
            construct_runtime! {
                pub enum Test where
                    Block = Block,
                    NodeBlock = Block,
                    UncheckedExtrinsic = UncheckedExtrinsic
                {
                    Instance1: pallet_collective::<Instance1>::{Pallet, Call, Storage} = 1,
                    Instance2: pallet_collective::<Instance2>::{Pallet, Call, Storage} = 2,
                    Balances: pallet_balances = 3
                }
            }
        };
        lint_macro(tokens).unwrap();
    }

    #[test]
    fn test_missing_pallet_parts() {
        let tokens = quote! {
            construct_runtime! {
                pub enum Test where
                    Block = Block,
                    NodeBlock = Block,
                    UncheckedExtrinsic = UncheckedExtrinsic
                {
                    PalletA = 0,
                    PalletB
                }
            }
        };
        lint_macro(tokens).unwrap_err();
    }

    #[test]
    fn test_with_expanded_pallet() {
        let tokens = quote! {
            construct_runtime! {
                pub enum Test where
                    Block = Block,
                    NodeBlock = Block,
                    UncheckedExtrinsic = UncheckedExtrinsic
                {
                    ExpandedPallet: pallet_balances expanded::{Pallet, Call, Storage} = 1,
                    RegularPallet: pallet_sudo = 2
                }
            }
        };
        lint_macro(tokens).unwrap();
    }

    #[test]
    fn test_with_no_pallets() {
        let tokens = quote! {
            construct_runtime! {
                pub enum Test where
                    Block = Block,
                    NodeBlock = Block,
                    UncheckedExtrinsic = UncheckedExtrinsic
                {
                }
            }
        };
        lint_macro(tokens).unwrap();
    }

    #[test]
    fn test_with_pallet_alias() {
        let tokens = quote! {
            construct_runtime! {
                pub enum Test where
                    Block = Block,
                    NodeBlock = Block,
                    UncheckedExtrinsic = UncheckedExtrinsic
                {
                    MyAlias: pallet_balances = 1,
                    OtherAlias: pallet_timestamp = 2
                }
            }
        };
        lint_macro(tokens).unwrap();
    }
}
