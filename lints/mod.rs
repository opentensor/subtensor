use syn::{File, Result};

pub mod lint;
pub use lint::*;

mod dummy_lint;
pub use dummy_lint::DummyLint;
