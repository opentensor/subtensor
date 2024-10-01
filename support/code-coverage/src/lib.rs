use proc_macro2::TokenStream as TokenStream2;
use procedural_fork::exports::pallet::parse::Def;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};
use syn::{visit::Visit, File, ItemMod};

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct PalletCoverageInfo {
    pub path: PathBuf,
    pub extrinsics: HashMap<String, usize>,
}

pub fn try_parse_pallet(item_mod: &ItemMod) -> Option<Def> {
    let pallet: Def = if let Ok(pallet) = Def::try_from(item_mod.clone(), false) {
        pallet
    } else {
        let Ok(pallet) = Def::try_from(item_mod.clone(), true) else {
            return None;
        };
        pallet
    };
    Some(pallet)
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
    let mut infos = Vec::new();
    PalletVisitor::for_each_pallet(&file, |item_mod, pallet: &Def| {
        let mut info = PalletCoverageInfo::default();
        info.path = path.to_path_buf();
        infos.push(info);
    });
    infos
}

#[derive(Default)]
pub struct PalletVisitor {
    pub pallets: Vec<(ItemMod, Def)>,
}

impl PalletVisitor {
    pub fn for_each_pallet<F>(file: &File, mut f: F) -> Self
    where
        F: FnMut(&ItemMod, &Def),
    {
        let mut visitor = PalletVisitor::default();
        visitor.visit_file(file);
        for (item_mod, pallet) in &visitor.pallets {
            f(item_mod, pallet);
        }
        visitor
    }
}

impl<'ast> Visit<'ast> for PalletVisitor {
    fn visit_item_mod(&mut self, item_mod: &'ast ItemMod) {
        let Some(pallet) = try_parse_pallet(item_mod) else {
            syn::visit::visit_item_mod(self, item_mod);
            return;
        };
        self.pallets.push((item_mod.clone(), pallet));
    }
}
