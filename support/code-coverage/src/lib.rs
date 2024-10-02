use proc_macro2::TokenStream as TokenStream2;
use procedural_fork::{exports::pallet::parse::Def, simulate_manifest_dir};
use quote::ToTokens;
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

pub fn try_parse_pallet(item_mod: &ItemMod, file_path: &Path) -> Option<Def> {
    simulate_manifest_dir("pallets/subtensor", || -> Option<Def> {
        if item_mod.content.is_none() || item_mod.ident != "pallet" {
            build_print::info!(
                "Skipping blank or irrelevant module: {}",
                item_mod.ident.to_string()
            );
            return None;
        }
        let item_mod = item_mod.clone();
        // manually import foreign sections defined by the `#[import_section]` attribute
        for attr in item_mod.attrs.iter() {
            if attr.meta.path().segments.last().unwrap().ident == "import_section" {
                build_print::note!("Importing section: {}", attr.to_token_stream().to_string());
                build_print::note!("path: {:?}", file_path);
                // TODO: in parallel, recursively search all files in the parent dir of `file_path` until we
                // find a module with a `#[pallet_section]` attribute whose name also matches
                // that of the `#[import_section]` attribute
            }
        }
        build_print::info!("Parsing module: {}", item_mod.ident.to_string());
        if let Ok(pallet) = Def::try_from(item_mod.clone(), false) {
            Some(pallet)
        } else if let Ok(pallet) = Def::try_from(item_mod.clone(), true) {
            Some(pallet)
        } else {
            let err = match Def::try_from(item_mod.clone(), false) {
                Err(e) => e,
                Ok(_) => unreachable!(),
            };
            build_print::error!("Error parsing pallet: {}", err);
            None
        }
    })
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
    build_print::info!("Analyzing file: {}", path.display());
    PalletVisitor::for_each_pallet(&file, &path, |_item_mod, _pallet: &Def| {
        let mut info = PalletCoverageInfo::default();
        info.path = path.to_path_buf();
        infos.push(info);
    });
    infos
}

#[derive(Default)]
pub struct PalletVisitor {
    pub pallets: Vec<(ItemMod, Def)>,
    pub file_path: PathBuf,
}

impl PalletVisitor {
    pub fn for_each_pallet<F>(file: &File, file_path: &Path, mut f: F) -> Self
    where
        F: FnMut(&ItemMod, &Def),
    {
        let mut visitor = PalletVisitor {
            pallets: Vec::new(),
            file_path: file_path.to_path_buf(),
        };
        visitor.visit_file(file);
        for (item_mod, pallet) in &visitor.pallets {
            f(item_mod, pallet);
        }
        visitor
    }
}

impl<'ast> Visit<'ast> for PalletVisitor {
    fn visit_item_mod(&mut self, item_mod: &'ast ItemMod) {
        if let Some(pallet) = try_parse_pallet(item_mod, &self.file_path) {
            self.pallets.push((item_mod.clone(), pallet));
        }
        syn::visit::visit_item_mod(self, item_mod);
    }
}
