use super::*;

pub struct RequireFreezeStruct;

impl Lint for RequireFreezeStruct {
    fn lint(_source: &File) -> Result<()> {
        Ok(())
    }
}
