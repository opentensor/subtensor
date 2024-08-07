use syn::{File, Result};

pub mod lint;
pub use lint::*;

mod dummy_lint;
mod require_freeze_struct;

pub use dummy_lint::DummyLint;
pub use require_freeze_struct::RequireFreezeStruct;

#[derive(Copy, Clone, Debug)]
pub struct SpanLocation {
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
}

impl Default for SpanLocation {
    fn default() -> Self {
        Self {
            start_line: 1,
            start_col: 0,
            end_line: 1,
            end_col: 0,
        }
    }
}

pub trait SpanHack {
    fn location(&self) -> SpanLocation;
}

impl SpanHack for proc_macro2::Span {
    fn location(&self) -> SpanLocation {
        //println!("{:#?}", self);
        let start = self.start();
        let end = self.end();
        SpanLocation {
            start_line: start.line,
            start_col: start.column,
            end_line: end.line,
            end_col: end.column,
        }
    }
}
