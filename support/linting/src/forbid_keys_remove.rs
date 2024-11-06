use super::*;
use syn::{
    punctuated::Punctuated, spanned::Spanned, token::Comma, visit::Visit, Expr, ExprCall, ExprPath,
    File,
};

pub struct ForbidKeysRemoveCall;

impl Lint for ForbidKeysRemoveCall {
    fn lint(source: &File) -> Result {
        let mut visitor = KeysRemoveVisitor::default();
        visitor.visit_file(source);

        if visitor.errors.is_empty() {
            Ok(())
        } else {
            Err(visitor.errors)
        }
    }
}

#[derive(Default)]
struct KeysRemoveVisitor {
    errors: Vec<syn::Error>,
}

impl<'ast> Visit<'ast> for KeysRemoveVisitor {
    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        let ExprCall { func, args, .. } = node;
        if is_keys_remove_call(func, args) {
            let msg = "Keys::<T>::remove()` is banned to prevent accidentally breaking \
                the neuron sequence. If you need to replace neuron, try `SubtensorModule::replace_neuron()`";
            self.errors.push(syn::Error::new(node.func.span(), msg));
        }
    }
}

fn is_keys_remove_call(func: &Expr, args: &Punctuated<Expr, Comma>) -> bool {
    let Expr::Path(ExprPath { path, .. }) = func else {
        return false;
    };
    let func = &path.segments;
    if func.len() != 2 || args.len() != 2 {
        return false;
    }

    func[0].ident == "Keys"
        && !func[0].arguments.is_none()
        && func[1].ident == "remove"
        && func[1].arguments.is_none()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lint(input: &str) -> Result {
        let mut visitor = KeysRemoveVisitor::default();
        visitor
            .visit_expr_call(&syn::parse_str(input).expect("should only use on a function call"));

        if visitor.errors.is_empty() {
            Ok(())
        } else {
            Err(visitor.errors)
        }
    }

    #[test]
    fn test_keys_remove_forbidden() {
        let input = r#"Keys::<T>::remove(netuid, uid_to_replace)"#;
        assert!(lint(input).is_err());
        let input = r#"Keys::<U>::remove(netuid, uid_to_replace)"#;
        assert!(lint(input).is_err());
        let input = r#"Keys::<U>::remove(1, "2".parse().unwrap(),)"#;
        assert!(lint(input).is_err());
    }

    #[test]
    fn test_non_keys_remove_not_forbidden() {
        let input = r#"remove(netuid, uid_to_replace)"#;
        assert!(lint(input).is_ok());
        let input = r#"Keys::remove(netuid, uid_to_replace)"#;
        assert!(lint(input).is_ok());
        let input = r#"Keys::<T>::remove::<U>(netuid, uid_to_replace)"#;
        assert!(lint(input).is_ok());
        let input = r#"Keys::<T>::remove(netuid, uid_to_replace, third_wheel)"#;
        assert!(lint(input).is_ok());
        let input = r#"ParentKeys::remove(netuid, uid_to_replace)"#;
        assert!(lint(input).is_ok());
        let input = r#"ChildKeys::<T>::remove(netuid, uid_to_replace)"#;
        assert!(lint(input).is_ok());
    }
}
