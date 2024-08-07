use super::*;
use syn::parse_quote;
use syn::punctuated::Punctuated;
use syn::{visit::Visit, Attribute, ItemStruct, Meta, MetaList, Path, Result, Token};

pub struct RequireFreezeStruct;

impl Lint for RequireFreezeStruct {
    fn lint(source: &syn::File) -> Result<()> {
        let mut visitor = EncodeDecodeVisitor::default();
        visitor.visit_file(source);

        if !visitor.errors.is_empty() {
            for error in visitor.errors {
                return Err(error);
            }
        }

        Ok(())
    }
}

#[derive(Default)]
struct EncodeDecodeVisitor {
    errors: Vec<syn::Error>,
}

impl<'ast> Visit<'ast> for EncodeDecodeVisitor {
    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        let has_encode_decode = node.attrs.iter().any(|attr| {
            let result = is_derive_encode_or_decode(attr);
            result
        });
        let has_freeze_struct = node.attrs.iter().any(|attr| {
            let result = is_freeze_struct(attr);
            result
        });

        if has_encode_decode && !has_freeze_struct {
            self.errors.push(syn::Error::new_spanned(
                &node.ident,
                "Struct with Encode/Decode derive must also have #[freeze_struct(..)] attribute.",
            ));
        }

        syn::visit::visit_item_struct(self, node);
    }
}

fn is_freeze_struct(attr: &Attribute) -> bool {
    if let Meta::List(meta_list) = &attr.meta {
        if meta_list.path.is_ident("freeze_struct") && !meta_list.tokens.is_empty() {
            return true;
        }
    }
    false
}

fn is_derive_encode_or_decode(attr: &Attribute) -> bool {
    if let Meta::List(MetaList { path, tokens, .. }) = &attr.meta {
        if path.is_ident("derive") {
            let nested: Punctuated<Path, Token![,]> = parse_quote!(#tokens);
            return nested.iter().any(|nested| {
                nested.segments.iter().any(|seg| seg.ident == "Encode")
                    || nested.segments.iter().any(|seg| seg.ident == "Decode")
            });
        }
    }
    false
}
