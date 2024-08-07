use syn::{File, Result};

pub mod lint;
pub use lint::*;

mod dummy_lint;
mod require_freeze_struct;

pub use dummy_lint::DummyLint;
pub use require_freeze_struct::RequireFreezeStruct;
