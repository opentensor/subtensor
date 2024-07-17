use super::*;

pub struct DummyLint;

impl Lint for DummyLint {
    fn lint(_source: &File) -> Result<()> {
        // This is a dummy lint that does nothing
        Ok(())
    }
}
