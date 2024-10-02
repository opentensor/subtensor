use proc_macro2::TokenStream as TokenStream2;
use procedural_fork::{exports::pallet::parse::Def, simulate_manifest_dir};
use quote::ToTokens;
use std::{
    collections::HashMap,
    fs::{self},
    path::{Path, PathBuf},
    str::FromStr,
};
use syn::{visit::Visit, Attribute, File, Ident, ItemMod};
use walkdir::WalkDir;

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct PalletCoverageInfo {
    pub path: PathBuf,
    pub extrinsics: HashMap<String, usize>,
}

pub fn try_parse_pallet(item_mod: &ItemMod, file_path: &Path) -> Option<Def> {
    simulate_manifest_dir("pallets/subtensor", || -> Option<Def> {
        // Single check for both content and identifier relevance
        if item_mod.content.is_none() || item_mod.ident != "pallet" {
            build_print::info!(
                "Skipping irrelevant or blank module: {}",
                item_mod.ident.to_string()
            );
            return None;
        }

        // manually import foreign sections defined by the `#[import_section]` attribute
        for attr in item_mod.attrs.iter() {
            if attr.meta.path().segments.last().unwrap().ident != "import_section" {
                continue;
            }
            build_print::note!("Importing section: {}", attr.to_token_stream().to_string());
            build_print::note!("path: {:?}", file_path);

            // Extract the section name from the attribute's args
            let Ok(inner_path) = attr.parse_args::<syn::Path>() else {
                continue;
            };
            let section_name = &inner_path.segments.last().unwrap().ident;

            if let Some(matching_path) = find_matching_pallet_section(file_path, &section_name) {
                build_print::note!("Found matching pallet section at: {:?}", matching_path);
            } else {
                build_print::warn!(
                    "Could not find a matching pallet section for: {}",
                    section_name
                );
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

fn find_matching_pallet_section(src_path: &Path, section_name: &Ident) -> Option<PathBuf> {
    let Some(base_path) = src_path.parent() else {
        return None;
    };
    for entry in WalkDir::new(base_path.parent().unwrap())
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if path == src_path {
            continue;
        }
        if path.is_file() {
            build_print::warn!("Checking file: {}", path.display());
            let Ok(content) = fs::read_to_string(path) else {
                continue;
            };
            let Ok(file) = syn::parse_file(&content) else {
                continue;
            };
            for item in file.items {
                let syn::Item::Mod(item_mod) = item else {
                    continue;
                };
                if item_mod.ident != *section_name {
                    continue;
                }
                build_print::note!("Checking module: {}", item_mod.ident.to_string());
                if item_mod.attrs.iter().any(|attr| is_pallet_section(attr)) {
                    return Some(path.to_path_buf());
                }
            }
        }
    }
    None
}

fn is_pallet_section(attr: &Attribute) -> bool {
    attr.meta.path().segments.last().unwrap().ident != "pallet_section"
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
