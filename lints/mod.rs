use syn::{File, Result};

/// A trait that defines custom lints that can be run within our workspace.
///
/// Each lint is run in parallel on all Rust source files in the workspace. Within a lint you
/// can issue an error the same way you would in a proc macro, and otherwise return `Ok(())` if
/// there are no errors.
pub trait Lint {
    fn lint(source: &File) -> Result<()>;
}
