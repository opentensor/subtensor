//! Proc-macro implementation for `define_proxy_filters!`.
//!
//! This module provides parsing and code generation for a declarative DSL that
//! defines proxy filter rules as a single source of truth. From one definition,
//! it generates:
//! - `proxy_type_filter()` — the runtime filtering function used by `InstanceFilter::filter()`
//! - `get_all_proxy_filters()` — the Runtime API data function returning filter metadata
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    Expr, Ident, Result, Token,
    parse::{Parse, ParseStream},
};

// ============================================================================
// DSL AST types
// ============================================================================

/// Top-level input: pallets { ... } followed by filter rules
pub struct ProxyFilterInput {
    pub pallets: Vec<PalletDef>,
    pub rules: Vec<FilterRule>,
}

/// Maps a short pallet name to its RuntimeCall variant and module path
/// e.g. `Balances => (Balances, pallet_balances)`
pub struct PalletDef {
    pub name: Ident,
    pub runtime_variant: Ident,
    pub module: Ident,
}

/// A single filter rule for one ProxyType variant
pub struct FilterRule {
    pub proxy_type: Ident,
    pub kind: FilterKind,
}

pub enum FilterKind {
    AllowAll,
    DenyAll,
    Allow { calls: Vec<CallRef>, exceptions: Vec<CallRef> },
    Deny { calls: Vec<CallRef> },
    AllowConditional { calls: Vec<ConditionalCallRef> },
    AllowNested { calls: Vec<NestedCallRef> },
}

/// Reference to a specific call or wildcard pallet
pub enum CallRef {
    Wildcard(Ident),
    Specific(Ident, Ident),
}

/// Conditional call: `Pallet::call where (field) < LIMIT`
pub struct ConditionalCallRef {
    pub pallet: Ident,
    pub call: Ident,
    pub field: Ident,
    pub limit: Expr,
}

/// Nested call: `Pallet::call where nested(field) == TargetPallet::target_call`
pub struct NestedCallRef {
    pub pallet: Ident,
    pub call: Ident,
    pub field: Ident,
    pub target_pallet: Ident,
    pub target_call: Ident,
}

// ============================================================================
// Parsing
// ============================================================================

mod kw {
    syn::custom_keyword!(pallets);
    syn::custom_keyword!(allow_all);
    syn::custom_keyword!(deny_all);
    syn::custom_keyword!(allow);
    syn::custom_keyword!(deny);
    syn::custom_keyword!(allow_conditional);
    syn::custom_keyword!(allow_nested);
    syn::custom_keyword!(except);
    syn::custom_keyword!(nested);
}

impl Parse for ProxyFilterInput {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<kw::pallets>()?;
        let pallet_content;
        syn::braced!(pallet_content in input);
        let mut pallets = Vec::new();
        while !pallet_content.is_empty() {
            pallets.push(pallet_content.parse::<PalletDef>()?);
            if pallet_content.peek(Token![,]) {
                pallet_content.parse::<Token![,]>()?;
            }
        }

        let mut rules = Vec::new();
        while !input.is_empty() {
            rules.push(input.parse::<FilterRule>()?);
        }

        Ok(ProxyFilterInput { pallets, rules })
    }
}

impl Parse for PalletDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        input.parse::<Token![=>]>()?;
        let content;
        syn::parenthesized!(content in input);
        let runtime_variant: Ident = content.parse()?;
        content.parse::<Token![,]>()?;
        let module: Ident = content.parse()?;
        Ok(PalletDef { name, runtime_variant, module })
    }
}

impl Parse for FilterRule {
    fn parse(input: ParseStream) -> Result<Self> {
        let proxy_type: Ident = input.parse()?;
        input.parse::<Token![=>]>()?;

        let kind = if input.peek(kw::allow_all) {
            input.parse::<kw::allow_all>()?;
            input.parse::<Token![;]>()?;
            FilterKind::AllowAll
        } else if input.peek(kw::deny_all) {
            input.parse::<kw::deny_all>()?;
            input.parse::<Token![;]>()?;
            FilterKind::DenyAll
        } else if input.peek(kw::allow_conditional) {
            input.parse::<kw::allow_conditional>()?;
            let content;
            syn::braced!(content in input);
            let calls = parse_conditional_calls(&content)?;
            FilterKind::AllowConditional { calls }
        } else if input.peek(kw::allow_nested) {
            input.parse::<kw::allow_nested>()?;
            let content;
            syn::braced!(content in input);
            let calls = parse_nested_calls(&content)?;
            FilterKind::AllowNested { calls }
        } else if input.peek(kw::allow) {
            input.parse::<kw::allow>()?;
            let content;
            syn::braced!(content in input);
            let calls = parse_call_refs(&content)?;

            let exceptions = if input.peek(kw::except) {
                input.parse::<kw::except>()?;
                let exc_content;
                syn::braced!(exc_content in input);
                parse_call_refs(&exc_content)?
            } else {
                Vec::new()
            };

            FilterKind::Allow { calls, exceptions }
        } else if input.peek(kw::deny) {
            input.parse::<kw::deny>()?;
            let content;
            syn::braced!(content in input);
            let calls = parse_call_refs(&content)?;
            FilterKind::Deny { calls }
        } else {
            return Err(input.error("expected allow_all, deny_all, allow, deny, allow_conditional, or allow_nested"));
        };

        // Consume optional trailing semicolon after braced rules
        if input.peek(Token![;]) {
            input.parse::<Token![;]>()?;
        }

        Ok(FilterRule { proxy_type, kind })
    }
}

fn parse_call_refs(input: ParseStream) -> Result<Vec<CallRef>> {
    let mut calls = Vec::new();
    while !input.is_empty() {
        let pallet: Ident = input.parse()?;
        input.parse::<Token![::]>()?;
        if input.peek(Token![*]) {
            input.parse::<Token![*]>()?;
            calls.push(CallRef::Wildcard(pallet));
        } else {
            let call: Ident = input.parse()?;
            calls.push(CallRef::Specific(pallet, call));
        }
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }
    }
    Ok(calls)
}

fn parse_conditional_calls(input: ParseStream) -> Result<Vec<ConditionalCallRef>> {
    let mut calls = Vec::new();
    while !input.is_empty() {
        let pallet: Ident = input.parse()?;
        input.parse::<Token![::]>()?;
        let call: Ident = input.parse()?;
        // parse: where (field) < LIMIT
        input.parse::<Token![where]>()?;
        let field_content;
        syn::parenthesized!(field_content in input);
        let field: Ident = field_content.parse()?;
        input.parse::<Token![<]>()?;
        let limit: Expr = input.parse()?;
        calls.push(ConditionalCallRef { pallet, call, field, limit });
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }
    }
    Ok(calls)
}

fn parse_nested_calls(input: ParseStream) -> Result<Vec<NestedCallRef>> {
    let mut calls = Vec::new();
    while !input.is_empty() {
        let pallet: Ident = input.parse()?;
        input.parse::<Token![::]>()?;
        let call: Ident = input.parse()?;
        // parse: where nested(field) == TargetPallet::target_call
        input.parse::<Token![where]>()?;
        input.parse::<kw::nested>()?;
        let field_content;
        syn::parenthesized!(field_content in input);
        let field: Ident = field_content.parse()?;
        input.parse::<Token![==]>()?;
        let target_pallet: Ident = input.parse()?;
        input.parse::<Token![::]>()?;
        let target_call: Ident = input.parse()?;
        calls.push(NestedCallRef { pallet, call, field, target_pallet, target_call });
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }
    }
    Ok(calls)
}

// ============================================================================
// Code generation
// ============================================================================

impl ProxyFilterInput {
    pub fn generate(self) -> TokenStream2 {
        let filter_fn = self.generate_filter_fn();
        let data_fn = self.generate_data_fn();

        quote! {
            #filter_fn
            #data_fn
        }
    }

    fn find_pallet(&self, name: &Ident) -> &PalletDef {
        self.pallets.iter().find(|p| p.name == *name)
            .unwrap_or_else(|| panic!("Pallet '{}' not found in pallets block", name))
    }

    // ========================================================================
    // filter function generation
    // ========================================================================

    fn generate_filter_fn(&self) -> TokenStream2 {
        let arms: Vec<TokenStream2> = self.rules.iter().map(|rule| {
            let pt = &rule.proxy_type;
            match &rule.kind {
                FilterKind::AllowAll => quote! {
                    ProxyType::#pt => true,
                },
                FilterKind::DenyAll => quote! {
                    ProxyType::#pt => false,
                },
                FilterKind::Allow { calls, exceptions } => {
                    let patterns = self.call_refs_to_patterns(calls);
                    if exceptions.is_empty() {
                        quote! {
                            ProxyType::#pt => matches!(c, #(#patterns)|*),
                        }
                    } else {
                        let exc_patterns = self.call_refs_to_patterns(exceptions);
                        quote! {
                            ProxyType::#pt => {
                                matches!(c, #(#patterns)|*) && !matches!(c, #(#exc_patterns)|*)
                            }
                        }
                    }
                }
                FilterKind::Deny { calls } => {
                    let patterns = self.call_refs_to_patterns(calls);
                    quote! {
                        ProxyType::#pt => !matches!(c, #(#patterns)|*),
                    }
                }
                FilterKind::AllowConditional { calls } => {
                    let arms = calls.iter().map(|cond| {
                        let pallet_def = self.find_pallet(&cond.pallet);
                        let variant = &pallet_def.runtime_variant;
                        let module = &pallet_def.module;
                        let call_name = &cond.call;
                        let field = &cond.field;
                        let limit = &cond.limit;
                        quote! {
                            RuntimeCall::#variant(#module::Call::#call_name { #field, .. }) => {
                                *#field < #limit
                            }
                        }
                    });
                    quote! {
                        ProxyType::#pt => match c {
                            #(#arms)*
                            _ => false,
                        },
                    }
                }
                FilterKind::AllowNested { calls } => {
                    let arms = calls.iter().map(|nested| {
                        let pallet_def = self.find_pallet(&nested.pallet);
                        let variant = &pallet_def.runtime_variant;
                        let module = &pallet_def.module;
                        let call_name = &nested.call;
                        let field = &nested.field;
                        let target_pallet_def = self.find_pallet(&nested.target_pallet);
                        let target_variant = &target_pallet_def.runtime_variant;
                        let target_module = &target_pallet_def.module;
                        let target_call_name = &nested.target_call;
                        quote! {
                            RuntimeCall::#variant(#module::Call::#call_name { #field, .. }) => {
                                let inner_call: RuntimeCall = *#field.clone();
                                matches!(
                                    inner_call,
                                    RuntimeCall::#target_variant(#target_module::Call::#target_call_name { .. })
                                )
                            }
                        }
                    });
                    quote! {
                        ProxyType::#pt => match c {
                            #(#arms)*
                            _ => false,
                        },
                    }
                }
            }
        }).collect();

        quote! {
            fn proxy_type_filter(pt: &ProxyType, c: &RuntimeCall) -> bool {
                match pt {
                    #(#arms)*
                }
            }
        }
    }

    fn call_refs_to_patterns(&self, calls: &[CallRef]) -> Vec<TokenStream2> {
        calls.iter().map(|call_ref| {
            match call_ref {
                CallRef::Wildcard(pallet) => {
                    let pallet_def = self.find_pallet(pallet);
                    let variant = &pallet_def.runtime_variant;
                    quote! { RuntimeCall::#variant(..) }
                }
                CallRef::Specific(pallet, call) => {
                    let pallet_def = self.find_pallet(pallet);
                    let variant = &pallet_def.runtime_variant;
                    let module = &pallet_def.module;
                    quote! { RuntimeCall::#variant(#module::Call::#call { .. }) }
                }
            }
        }).collect()
    }

    // ========================================================================
    // data function generation
    // ========================================================================

    fn generate_data_fn(&self) -> TokenStream2 {
        let entries: Vec<TokenStream2> = self.rules.iter().map(|rule| {
            let pt = &rule.proxy_type;
            let (mode, calls_expr, exceptions_expr) = match &rule.kind {
                FilterKind::AllowAll => (
                    quote! { FilterMode::AllowAll },
                    quote! { Vec::new() },
                    quote! { Vec::new() },
                ),
                FilterKind::DenyAll => (
                    quote! { FilterMode::DenyAll },
                    quote! { Vec::new() },
                    quote! { Vec::new() },
                ),
                FilterKind::Allow { calls, exceptions } => (
                    quote! { FilterMode::Allow },
                    self.call_refs_to_data(calls),
                    self.call_refs_to_data(exceptions),
                ),
                FilterKind::Deny { calls } => (
                    quote! { FilterMode::Deny },
                    self.call_refs_to_data(calls),
                    quote! { Vec::new() },
                ),
                FilterKind::AllowConditional { calls } => {
                    let data = self.conditional_calls_to_data(calls);
                    (
                        quote! { FilterMode::Allow },
                        data,
                        quote! { Vec::new() },
                    )
                }
                FilterKind::AllowNested { calls } => {
                    let data = self.nested_calls_to_data(calls);
                    (
                        quote! { FilterMode::Allow },
                        data,
                        quote! { Vec::new() },
                    )
                }
            };

            quote! {
                ProxyFilterInfo {
                    proxy_type: ProxyType::#pt.into(),
                    name: alloc::format!("{:?}", ProxyType::#pt).into_bytes(),
                    deprecated: ProxyType::#pt.is_deprecated(),
                    filter_mode: #mode,
                    calls: #calls_expr,
                    exceptions: #exceptions_expr,
                }
            }
        }).collect();

        quote! {
            pub fn get_all_proxy_filters() -> Vec<ProxyFilterInfo> {
                vec![
                    #(#entries),*
                ]
            }
        }
    }

    fn call_refs_to_data(&self, calls: &[CallRef]) -> TokenStream2 {
        let items: Vec<TokenStream2> = calls.iter().map(|call_ref| {
            match call_ref {
                CallRef::Wildcard(pallet) => {
                    let pallet_def = self.find_pallet(pallet);
                    let runtime_variant = &pallet_def.runtime_variant;
                    quote! { pallet_wildcard::<#runtime_variant>() }
                }
                CallRef::Specific(pallet, call) => {
                    let pallet_def = self.find_pallet(pallet);
                    let runtime_variant = &pallet_def.runtime_variant;
                    let module = &pallet_def.module;
                    let call_str = call.to_string();
                    quote! {
                        call_info_by_name::<#runtime_variant, #module::Call<Runtime>>(#call_str)
                    }
                }
            }
        }).collect();
        quote! { vec![#(#items),*] }
    }

    fn conditional_calls_to_data(&self, calls: &[ConditionalCallRef]) -> TokenStream2 {
        let items: Vec<TokenStream2> = calls.iter().map(|cond| {
            let pallet_def = self.find_pallet(&cond.pallet);
            let runtime_variant = &pallet_def.runtime_variant;
            let module = &pallet_def.module;
            let call_str = cond.call.to_string();
            let field_str = cond.field.to_string();
            let limit = &cond.limit;
            quote! {
                call_info_by_name_conditional::<#runtime_variant, #module::Call<Runtime>>(
                    #call_str,
                    CallCondition::ParamLessThan {
                        param_name: #field_str.as_bytes().to_vec(),
                        limit: Into::<u64>::into(#limit) as u128,
                    },
                )
            }
        }).collect();
        quote! { vec![#(#items),*] }
    }

    fn nested_calls_to_data(&self, calls: &[NestedCallRef]) -> TokenStream2 {
        let items: Vec<TokenStream2> = calls.iter().map(|nested| {
            let pallet_def = self.find_pallet(&nested.pallet);
            let runtime_variant = &pallet_def.runtime_variant;
            let module = &pallet_def.module;
            let call_str = nested.call.to_string();
            let target_pallet_def = self.find_pallet(&nested.target_pallet);
            let target_runtime_variant = &target_pallet_def.runtime_variant;
            let target_call_str = nested.target_call.to_string();
            quote! {
                call_info_by_name_conditional::<#runtime_variant, #module::Call<Runtime>>(
                    #call_str,
                    CallCondition::NestedCallMustBe {
                        pallet_name: <#target_runtime_variant as PalletInfoAccess>::name().as_bytes().to_vec(),
                        call_name: #target_call_str.as_bytes().to_vec(),
                    },
                )
            }
        }).collect();
        quote! { vec![#(#items),*] }
    }
}
