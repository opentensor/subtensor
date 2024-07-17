use syn::{File, Result};

pub mod lint;
pub use lint::*;

mod dummy_lint;
use dummy_lint::DummyLint;
