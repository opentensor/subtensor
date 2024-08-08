use proc_macro2::TokenStream;

use super::*;

/// A trait that defines custom lints that can be run within our workspace.
///
/// Each lint is run in parallel on all Rust source files in the workspace. Within a lint you
/// can issue an error the same way you would in a proc macro, and otherwise return `Ok(())` if
/// there are no errors.
pub trait Lint: Send + Sync {
    /// Lints the given Rust source file, returning a compile error if any issues are found.
    fn lint(source: &TokenStream) -> Result<()>;
}
