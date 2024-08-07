use proc_macro2::TokenStream;

use super::*;

pub struct DummyLint;

impl Lint for DummyLint {
    fn lint(_source: &TokenStream) -> Result<()> {
        Ok(())
    }
}
