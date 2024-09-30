use proc_macro2::TokenStream as TokenStream2;
use procedural_fork::exports::pallet::parse::Def;
use quote::ToTokens;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};
use syn::{parse2, spanned::Spanned, File, Item};

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct PalletCoverageInfo {
    pub path: PathBuf,
    pub extrinsics: HashMap<String, usize>,
}

pub fn analyze_file(path: &Path) -> Vec<PalletCoverageInfo> {
    let Ok(content) = fs::read_to_string(path) else {
        return Vec::new();
    };
    let Ok(parsed_tokens) = TokenStream2::from_str(&content) else {
        return Vec::new();
    };
    let Ok(file) = syn::parse2::<syn::File>(parsed_tokens) else {
        return Vec::new();
    };
    // TODO: use a visitor here instead
    for item in &file.items {
        let Item::Mod(item_mod) = item else { continue };
        let pallet: Def = if let Ok(pallet) = Def::try_from(item_mod.clone(), false) {
            pallet
        } else {
            let Ok(pallet) = Def::try_from(item_mod.clone(), true) else {
                continue;
            };
            pallet
        };
        let mut info = PalletCoverageInfo::default();
        info.path = path.to_path_buf();
    }
    todo!()
}
