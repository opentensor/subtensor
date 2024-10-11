use super::*;
use syn::{spanned::Spanned, visit::Visit};

pub struct DisallowV1Benchmarks;

impl Lint for DisallowV1Benchmarks {
    fn lint(source: &syn::File) -> Result {
        let mut visitor = ExprMacroVisitor::new();
        visitor.visit_file(source);

        if !visitor.errors.is_empty() {
            return Err(visitor.errors);
        }

        Ok(())
    }
}

struct ExprMacroVisitor {
    errors: Vec<syn::Error>,
}

impl<'ast> syn::visit::Visit<'ast> for ExprMacroVisitor {
    fn visit_expr_macro(&mut self, i: &'ast syn::ExprMacro) {
        let Some(segment) = i.mac.path.segments.last() else {
            return;
        };

        if segment.ident == "benchmarks" {
            self.errors.push(syn::Error::new(
                segment.span(),
                "V1 benchmark syntax is disallowed!".to_owned(),
            ));
        }
    }
}

impl ExprMacroVisitor {
    fn new() -> Self {
        Self { errors: Vec::new() }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use quote::quote;

    fn lint_macro(input: proc_macro2::TokenStream) -> Result {
        let expr_macro: syn::ExprMacro = syn::parse2(input).unwrap();
        let mut visitor = ExprMacroVisitor::new();
        visitor.visit_expr_macro(&expr_macro);
        if !visitor.errors.is_empty() {
            return Err(visitor.errors);
        }
        Ok(())
    }

    #[test]
    fn test_disallow_benchmarks_v1() {
        let input = quote! {
            benchmarks! {
                benchmark_test {

                }: _(RawOrigin::Root)
            }
        };

        lint_macro(input).unwrap_err();
    }
}
