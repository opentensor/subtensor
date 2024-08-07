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
    fn location(&self, source: &str) -> SpanLocation;
}

impl SpanHack for proc_macro2::Span {
    fn location(&self, source: &str) -> SpanLocation {
        let range = self.byte_range();

        let mut start_line = 1;
        let mut start_col = 0;
        let mut end_line = 1;
        let mut end_col = 0;
        let mut current_col = 0;

        for (i, c) in source.chars().enumerate() {
            if i == range.start {
                start_line = end_line;
                start_col = current_col;
            }
            if i == range.end {
                end_line = end_line;
                end_col = current_col;
                break;
            }
            if c == '\n' {
                current_col = 0;
                end_line += 1;
            } else {
                current_col += 1;
            }
        }

        SpanLocation {
            start_line,
            start_col,
            end_line,
            end_col,
        }
    }
}
