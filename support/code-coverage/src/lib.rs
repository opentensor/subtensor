use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use syn::File;

pub struct PalletCoverageInfo {
    pub pallet_name: String,
    pub path: PathBuf,
    pub extrinsics: HashMap<String, usize>,
    pub events: HashMap<String, usize>,
    pub hooks: HashMap<String, usize>,
    pub calls: HashMap<String, usize>,
    pub storage: HashMap<String, usize>,
}
