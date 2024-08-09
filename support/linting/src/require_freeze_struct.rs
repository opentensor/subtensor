use super::*;
use proc_macro2::TokenStream;
use syn::{
    parse_quote, punctuated::Punctuated, visit::Visit, Attribute, ItemStruct, Meta, MetaList, Path,
    Result, Token,
};

pub struct RequireFreezeStruct;

fn meh() {}

impl Lint for RequireFreezeStruct {
    fn lint(source: &TokenStream) -> Result<()> {
        let mut visitor = EncodeDecodeVisitor::default();

        let file = syn::parse2::<syn::File>(source.clone()).unwrap();
        visitor.visit_file(&file);

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
        let has_encode_decode = node
            .attrs
            .iter()
            .any(|attr| is_derive_encode_or_decode(attr));
        let has_freeze_struct = node.attrs.iter().any(|attr| is_freeze_struct(attr));

        if has_encode_decode && !has_freeze_struct {
            self.errors.push(syn::Error::new(
                node.ident.span(),
                "Struct with Encode/Decode derive must also have #[freeze_struct(..)] attribute.",
            ));
        }

        syn::visit::visit_item_struct(self, node);
    }
}

fn is_freeze_struct(attr: &Attribute) -> bool {
    if let Meta::List(meta_list) = &attr.meta {
        let Some(seg) = meta_list.path.segments.last() else {
            return false;
        };
        if seg.ident == "freeze_struct" && !meta_list.tokens.is_empty() {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn lint_struct(input: &str) -> Result<()> {
        let item_struct: ItemStruct = syn::parse_str(input).unwrap();
        let mut visitor = EncodeDecodeVisitor::default();
        visitor.visit_item_struct(&item_struct);
        if visitor.errors.is_empty() {
            Ok(())
        } else {
            Err(visitor.errors[0].clone())
        }
    }

    #[test]
    fn test_no_attributes() {
        let input = r#"
            pub struct Test {
                field: u32,
            }
        "#;
        assert!(lint_struct(input).is_ok());
    }

    #[test]
    fn test_freeze_struct_only() {
        let input = r#"
            #[freeze_struct("12345")]
            pub struct Test {
                field: u32,
            }
        "#;
        assert!(lint_struct(input).is_ok());
    }

    #[test]
    fn test_encode_only() {
        let input = r#"
            #[derive(Encode)]
            pub struct Test {
                field: u32,
            }
        "#;
        assert!(lint_struct(input).is_err());
    }

    #[test]
    fn test_decode_only() {
        let input = r#"
            #[derive(Decode)]
            pub struct Test {
                field: u32,
            }
        "#;
        assert!(lint_struct(input).is_err());
    }

    #[test]
    fn test_encode_and_freeze_struct() {
        let input = r#"
            #[freeze_struct("12345")]
            #[derive(Encode)]
            pub struct Test {
                field: u32,
            }
        "#;
        assert!(lint_struct(input).is_ok());
    }

    #[test]
    fn test_decode_and_freeze_struct() {
        let input = r#"
            #[freeze_struct("12345")]
            #[derive(Decode)]
            pub struct Test {
                field: u32,
            }
        "#;
        assert!(lint_struct(input).is_ok());
    }

    #[test]
    fn test_encode_decode_without_freeze_struct() {
        let input = r#"
            #[derive(Encode, Decode)]
            pub struct Test {
                field: u32,
            }
        "#;
        assert!(lint_struct(input).is_err());
    }

    #[test]
    fn test_encode_decode_with_freeze_struct() {
        let input = r#"
            #[freeze_struct("12345")]
            #[derive(Encode, Decode)]
            pub struct Test {
                field: u32,
            }
        "#;
        assert!(lint_struct(input).is_ok());
    }

    #[test]
    fn test_temporary_freeze_struct() {
        let input = r#"
            #[freeze_struct]
            #[derive(Encode, Decode)]
            pub struct Test {
                field: u32,
            }
        "#;
        assert!(lint_struct(input).is_err());
    }
}