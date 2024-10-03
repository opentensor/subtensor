use build_print::*;
use proc_macro2::TokenStream as TokenStream2;
use procedural_fork::{exports::pallet::parse::Def, simulate_manifest_dir};
use quote::ToTokens;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{
    collections::HashMap,
    ffi::OsStr,
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
    PalletVisitor::for_each_pallet(&file, &path, |_item_mod, _pallet: &Def| {
        let mut info = PalletCoverageInfo::default();
        info.path = path.to_path_buf();
        infos.push(info);
    });
    infos
}

pub fn try_parse_pallet(item_mod: &ItemMod, file_path: &Path) -> Option<Def> {
    simulate_manifest_dir("pallets/subtensor", || -> Option<Def> {
        // skip non-inline modules
        let mut item_mod = item_mod.clone();
        let Some((_, ref mut content)) = item_mod.content else {
            return None;
        };

        // skip non-pallet modules
        if item_mod.ident != "pallet" {
            return None;
        }

        let mut section_announced = false;

        // manually import foreign sections defined by the `#[import_section]` attribute
        for attr in item_mod.attrs.iter() {
            if attr.meta.path().segments.last().unwrap().ident != "import_section" {
                continue;
            }

            // Extract the section name from the attribute's args
            let Ok(inner_path) = attr.parse_args::<syn::Path>() else {
                continue;
            };
            let section_name = &inner_path.segments.last().unwrap().ident;

            if !section_announced {
                custom_println!(
                    "[code-coverage]",
                    cyan,
                    "importing pallet sections for '{}'...",
                    extract_pallet_name(file_path).unwrap_or("unknown".to_string()),
                );
                section_announced = true;
            }

            if let Some(section_mod) = find_matching_pallet_section(file_path, &section_name) {
                let Some((_, mut section_content)) = section_mod.content else {
                    continue;
                };
                content.append(&mut section_content);
                custom_println!("[code-coverage]", cyan, "└ imported '{}' ✔", section_name,);
            } else {
                custom_println!(
                    "[code-coverage]",
                    red,
                    "could not find matching section for: '{}'",
                    section_name,
                );
            }
        }

        let pallet = if let Ok(pallet) = Def::try_from(item_mod.clone(), false) {
            Some(pallet)
        } else if let Ok(pallet) = Def::try_from(item_mod.clone(), true) {
            Some(pallet)
        } else {
            let err = match Def::try_from(item_mod.clone(), false) {
                Err(e) => e,
                Ok(_) => unreachable!(),
            };
            custom_println!(
                "[code-coverage]",
                red,
                "unable to parse pallet in {}:",
                file_path.display()
            );
            custom_println!("[code-coverage]", red, "{}", err);
            None
        };
        match pallet {
            Some(pallet) => {
                custom_println!(
                    "[code-coverage]",
                    green,
                    "parsed pallet '{}' ✔ ({})",
                    extract_pallet_name(file_path).unwrap_or("unknown".to_string()),
                    file_path.display(),
                );
                Some(pallet)
            }
            None => None,
        }
    })
}

fn find_matching_pallet_section(src_path: &Path, section_name: &Ident) -> Option<ItemMod> {
    let Some(base_path) = src_path.parent() else {
        return None;
    };
    let rust_files = WalkDir::new(base_path.parent().unwrap())
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| {
            e.path() != src_path
                && e.path().is_file()
                && e.path().extension() == Some(OsStr::new("rs"))
        })
        .map(|e| e.path().to_path_buf())
        .collect::<Vec<PathBuf>>();
    let section_name = section_name.to_string().trim().to_string();
    rust_files
        .par_iter()
        .find_map_any(|path| {
            if !path.display().to_string().contains("macros") {
                return None;
            }
            let Ok(content) = fs::read_to_string(path) else {
                return None;
            };
            let Ok(file) = syn::parse_file(&content) else {
                return None;
            };
            for item in file.items {
                let syn::Item::Mod(item_mod) = item else {
                    continue;
                };
                if item_mod.ident != section_name {
                    continue;
                }
                if item_mod.attrs.iter().any(|attr| is_pallet_section(attr)) {
                    // can't move ItemMod across thread boundaries
                    return Some(item_mod.to_token_stream().to_string());
                }
            }
            None
        })
        .map(|s| syn::parse_str::<ItemMod>(&s).unwrap()) // can't move ItemMod across thread boundaries
}

fn is_pallet_section(attr: &Attribute) -> bool {
    attr.meta.path().segments.last().unwrap().ident != "pallet_section"
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

pub fn extract_pallet_name(path: &Path) -> Option<String> {
    // Try to get the parent directory, then the directory name
    path.parent()?
        .parent()? // Go up one level to the "pallets" directory
        .file_name() // Get the directory name "subtensor"
        .and_then(|os_str| os_str.to_str()) // Convert OsStr to &str
        .map(|s| s.to_string()) // Convert &str to String
}
