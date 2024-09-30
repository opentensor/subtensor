use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use syn::File;

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct PalletCoverageInfo {
    pub pallet_name: String,
    pub path: PathBuf,
    pub extrinsics: HashMap<String, usize>,
}

pub fn analyze_pallet(file: &File) -> PalletCoverageInfo {
    let mut pallet_coverage_info = PalletCoverageInfo::default();
    todo!()
}
