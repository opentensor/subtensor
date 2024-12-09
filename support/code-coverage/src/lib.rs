#![allow(clippy::unwrap_used)]
#![allow(clippy::arithmetic_side_effects)]

use build_print::*;
use proc_macro2::TokenStream as TokenStream2;
use procedural_fork::exports::pallet::parse::Def;
use quote::ToTokens;
use rayon::{
    iter::{IntoParallelRefIterator, ParallelIterator},
    slice::ParallelSliceMut,
};
use std::{
    collections::{HashMap, HashSet},
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};
use syn::{visit::Visit, Attribute, File, Ident, ItemFn, ItemMod};
use walkdir::WalkDir;

/// Code coverage information for a pallet
#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct PalletInfo {
    pub path: PathBuf,
    pub pallet_name: String,
    pub methods: Vec<String>,
}

/// Recursively collects all Rust files in the given directory
pub fn collect_rust_files(dir: &Path) -> Vec<PathBuf> {
    let mut rust_files = Vec::new();

    for entry in WalkDir::new(dir) {
        let Ok(entry) = entry else {
            continue;
        };
        let path = entry.path();

        // Skip any path that contains "target" directory
        if path.components().any(|component| {
            component.as_os_str() == "target" || component.as_os_str() == "procedural-fork"
        }) || path.ends_with("build.rs")
        {
            continue;
        }

        if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            rust_files.push(path.to_path_buf());
        }
    }

    rust_files
}

pub fn analyze_files(rust_files: &[PathBuf], workspace_root: &Path) -> Vec<PalletInfo> {
    custom_println!("[code-coverage]", cyan, "generating coverage report...");
    let start = std::time::Instant::now();
    custom_println!(
        "[code-coverage]",
        cyan,
        "searching {} rust files for pallets in parallel...",
        rust_files.len()
    );
    let infos = rust_files
        .par_iter()
        .map(|path| {
            if path.display().to_string().contains("test") {
                return Vec::new();
            }
            analyze_file(path, workspace_root)
        })
        .reduce(Vec::new, |mut acc, mut infos| {
            acc.append(&mut infos);
            acc
        });

    custom_println!(
        "[code-coverage]",
        cyan,
        "searching {} rust files for tests in parallel...",
        rust_files.len()
    );
    let tests = find_tests(rust_files);
    let benchmarks = find_benchmarks(rust_files);

    let methods = infos
        .iter()
        .flat_map(|i| i.methods.iter())
        .collect::<HashSet<_>>();

    custom_println!("[code-coverage]", green, "found {} tests", tests.len());
    custom_println!(
        "[code-coverage]",
        green,
        "found {} benchmarks",
        benchmarks.len()
    );

    custom_println!(
        "[code-coverage]",
        green,
        "found {} unique calls across {} pallets",
        methods.len(),
        infos.len(),
    );

    custom_println!("[code-coverage]", cyan, "compiling statistics...");

    let mut coverage: HashMap<String, usize> = HashMap::new();
    // this takes about 6ms serially so better to keep serial for now
    for method in &methods {
        coverage
            .entry(method.strip_prefix("sudo_").unwrap_or(method).to_string())
            .or_insert(0);
    }
    for test in &tests {
        for method in &test.method_calls {
            let method = method.strip_prefix("sudo_").unwrap_or(method);
            let Some(count) = coverage.get_mut(method) else {
                continue;
            };
            *count += 1;
        }
    }
    // if a call is in a benchmark, we can consider it tested since a benchmark test is
    // auto-generated
    for benchmark in &benchmarks {
        for call in &benchmark.calls {
            let call = call.strip_prefix("sudo_").unwrap_or(call);
            let Some(count) = coverage.get_mut(call) else {
                continue;
            };
            *count += 1;
        }
    }
    let mut coverage = coverage.into_iter().collect::<Vec<_>>();
    coverage.par_sort_by_key(|(_, v)| *v);

    let (covered, uncovered) = coverage.iter().partition::<Vec<_>, _>(|(_, v)| *v > 0);

    custom_println!(
        "[code-coverage]",
        cyan,
        "    total covered: {}",
        covered.len()
    );
    custom_println!(
        "[code-coverage]",
        cyan,
        "  total uncovered: {}",
        uncovered.len()
    );
    custom_println!(
        "[code-coverage]",
        cyan,
        "   total coverage: {:.2}%",
        covered.len() as f64 / methods.len() as f64 * 100.0
    );

    let finish = std::time::Instant::now();
    custom_println!(
        "[code-coverage]",
        green,
        "coverage report generated in {:?}",
        finish - start
    );
    infos
}

fn analyze_file(path: &Path, root_path: &Path) -> Vec<PalletInfo> {
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
    PalletVisitor::for_each_pallet(&file, path, root_path, |_item_mod, pallet: &Def| {
        custom_println!(
            "[code-coverage]",
            green,
            "parsed pallet '{}' ({})",
            extract_pallet_name(path).unwrap_or("unknown".to_string()),
            strip_common_suffix("/src/lib.rs".as_ref(), strip_common_prefix(root_path, path))
                .display(),
        );
        let mut info = PalletInfo {
            path: path.to_path_buf(),
            pallet_name: extract_pallet_name(path).unwrap_or("pallet".to_string()),
            ..Default::default()
        };

        // collect all Call methods
        if let Some(call) = &pallet.call {
            info.methods
                .append(&mut call.methods.iter().map(|m| m.name.to_string()).collect());
        }

        infos.push(info);
    });
    infos
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct TestInfo {
    pub path: PathBuf,
    pub name: String,
    pub method_calls: HashSet<String>,
}

/// Finds all tests in the given set of rust files, using a parallel map-reduce.
pub fn find_tests(rust_files: &[PathBuf]) -> Vec<TestInfo> {
    rust_files
        .par_iter()
        .map(|path| {
            let Ok(content) = fs::read_to_string(path) else {
                return Vec::new();
            };
            let Ok(file) = syn::parse_file(&content) else {
                return Vec::new();
            };
            let mut visitor = TestVisitor { tests: Vec::new() };
            visitor.visit_file(&file);
            visitor
                .tests
                .into_iter()
                .map(|f| {
                    let mut method_calls = HashSet::new();
                    let mut visitor = MethodCallVisitor {
                        method_calls: &mut method_calls,
                    };
                    visitor.visit_item_fn(&f);
                    TestInfo {
                        path: path.clone(),
                        name: f.sig.ident.to_string(),
                        method_calls,
                    }
                })
                .collect()
        })
        .reduce(Vec::new, |mut acc, mut infos| {
            acc.append(&mut infos);
            acc
        })
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkInfo {
    pub calls: HashSet<String>,
}

/// Finds all benchmarks in the given set of rust files, using a parallel map-reduce
pub fn find_benchmarks(rust_files: &[PathBuf]) -> Vec<BenchmarkInfo> {
    rust_files
        .par_iter()
        .map(|path| {
            let Ok(content) = fs::read_to_string(path) else {
                return Vec::new();
            };
            let Ok(file) = syn::parse_file(&content) else {
                return Vec::new();
            };
            let mut visitor = BenchmarkVisitor {
                benchmarks: Vec::new(),
            };
            visitor.visit_file(&file);
            visitor
                .benchmarks
                .into_iter()
                .map(|f| {
                    let mut calls = HashSet::new();
                    let mut visitor = CallVisitor { calls: &mut calls };
                    visitor.visit_item_fn(&f);
                    BenchmarkInfo { calls }
                })
                .collect()
        })
        .reduce(Vec::new, |mut acc, mut infos| {
            acc.append(&mut infos);
            acc
        })
}

pub struct CallVisitor<'a> {
    pub calls: &'a mut HashSet<String>,
}

impl<'ast> Visit<'ast> for CallVisitor<'_> {
    fn visit_expr_call(&mut self, i: &'ast syn::ExprCall) {
        if let syn::Expr::Path(expr) = &*i.func {
            if let Some(seg) = expr.path.segments.last() {
                self.calls.insert(seg.ident.to_string());
            }
        }
        syn::visit::visit_expr_call(self, i);
    }
}

pub struct MethodCallVisitor<'a> {
    pub method_calls: &'a mut HashSet<String>,
}

impl<'ast> Visit<'ast> for MethodCallVisitor<'_> {
    fn visit_expr_method_call(&mut self, i: &'ast syn::ExprMethodCall) {
        self.method_calls.insert(i.method.to_string());
        syn::visit::visit_expr_method_call(self, i);
    }

    fn visit_expr_path(&mut self, i: &'ast syn::ExprPath) {
        self.method_calls.insert(
            i.path
                .segments
                .last()
                .expect("at least one element is expected in path")
                .ident
                .to_string(),
        );
        syn::visit::visit_expr_path(self, i);
    }
}

pub struct TestVisitor {
    pub tests: Vec<ItemFn>,
}

impl<'ast> Visit<'ast> for TestVisitor {
    fn visit_item_fn(&mut self, item_fn: &'ast ItemFn) {
        if item_fn.attrs.iter().any(|attr| {
            let Some(seg) = attr.path().segments.last() else {
                return false;
            };
            seg.ident == "test" || seg.ident == "should_panic"
        }) {
            self.tests.push(item_fn.clone());
        }
        syn::visit::visit_item_fn(self, item_fn);
    }
}

pub struct BenchmarkVisitor {
    pub benchmarks: Vec<ItemFn>,
}

impl<'ast> Visit<'ast> for BenchmarkVisitor {
    fn visit_item_fn(&mut self, item_fn: &'ast ItemFn) {
        if item_fn.attrs.iter().any(|attr| {
            let Some(seg) = attr.path().segments.last() else {
                return false;
            };
            seg.ident == "benchmark"
        }) {
            self.benchmarks.push(item_fn.clone());
        }
        syn::visit::visit_item_fn(self, item_fn);
    }
}

/// Tries to parse a pallet from a module
pub fn try_parse_pallet(item_mod: &ItemMod, file_path: &Path, root_path: &Path) -> Option<Def> {
    // skip non-inline modules
    let mut item_mod = item_mod.clone();
    let (_, ref mut content) = item_mod.content.as_mut()?;

    // skip non-pallet modules
    if item_mod.ident != "pallet" {
        return None;
    }

    let mut section_announced = false;

    // manually import foreign sections defined by the `#[import_section]` attribute
    for attr in item_mod.attrs.iter() {
        if attr
            .meta
            .path()
            .segments
            .last()
            .expect("at least one element is expected in path")
            .ident
            != "import_section"
        {
            continue;
        }

        // Extract the section name from the attribute's args
        let Ok(inner_path) = attr.parse_args::<syn::Path>() else {
            continue;
        };
        let section_name = &inner_path
            .segments
            .last()
            .expect("at least one element is expected in path")
            .ident;

        if !section_announced {
            custom_println!(
                "[code-coverage]",
                cyan,
                "importing pallet sections for '{}' ({})...",
                extract_pallet_name(file_path).unwrap_or("unknown".to_string()),
                strip_common_suffix(
                    "/src/lib.rs".as_ref(),
                    strip_common_prefix(root_path, file_path)
                )
                .display(),
            );
            section_announced = true;
        }

        if let Some((section_mod, section_path)) =
            find_matching_pallet_section(file_path, section_name)
        {
            let Some((_, mut section_content)) = section_mod.content else {
                continue;
            };
            content.append(&mut section_content);
            custom_println!(
                "[code-coverage]",
                cyan,
                "└ imported '{}' ({})",
                section_name,
                strip_common_suffix(
                    "/src/lib.rs".as_ref(),
                    strip_common_prefix(file_path, &section_path)
                )
                .display()
            );
        } else {
            custom_println!(
                "[code-coverage]",
                red,
                "could not find matching section for: '{}'",
                section_name,
            );
        }
    }

    if let Ok(pallet) = Def::try_from(item_mod.clone(), false) {
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
    }
}

fn find_matching_pallet_section(
    src_path: &Path,
    section_name: &Ident,
) -> Option<(ItemMod, PathBuf)> {
    let base_path = src_path.parent()?;
    let rust_files = WalkDir::new(
        base_path
            .parent()
            .expect("the base path is not a top level directory"),
    )
    .into_iter()
    .filter_map(Result::ok)
    .filter(|e| {
        e.path() != src_path && e.path().is_file() && e.path().extension() == Some(OsStr::new("rs"))
    })
    .map(|e| e.path().to_path_buf())
    .collect::<Vec<PathBuf>>();
    let section_name = section_name.to_string().trim().to_string();
    rust_files
        .par_iter()
        .find_map_any(|path| {
            if path.display().to_string().contains("test") {
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
                if item_mod.attrs.iter().any(is_pallet_section) {
                    // can't move ItemMod across thread boundaries
                    return Some((item_mod.to_token_stream().to_string(), path.to_path_buf()));
                }
            }
            None
        })
        .map(|(s, p)| (syn::parse_str::<ItemMod>(&s).unwrap(), p)) // can't move ItemMod across thread boundaries
}

fn is_pallet_section(attr: &Attribute) -> bool {
    attr.meta.path().segments.last().unwrap().ident != "pallet_section"
}

/// A visitor that collects pallets from a file/module
#[derive(Default)]
pub struct PalletVisitor {
    pub pallets: Vec<(ItemMod, Def)>,
    pub file_path: PathBuf,
    pub workspace_root_path: PathBuf,
}

impl PalletVisitor {
    pub fn for_each_pallet<F>(file: &File, file_path: &Path, root_path: &Path, mut f: F) -> Self
    where
        F: FnMut(&ItemMod, &Def),
    {
        let mut visitor = PalletVisitor {
            pallets: Vec::new(),
            file_path: file_path.to_path_buf(),
            workspace_root_path: root_path.to_path_buf(),
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
        if let Some(pallet) = try_parse_pallet(item_mod, &self.file_path, &self.workspace_root_path)
        {
            self.pallets.push((item_mod.clone(), pallet));
        }
        syn::visit::visit_item_mod(self, item_mod);
    }
}

/// Extracts the pallet name from a path
pub fn extract_pallet_name(path: &Path) -> Option<String> {
    // Try to get the parent directory, then the directory name
    path.parent()?
        .parent()? // Go up one level to the "pallets" directory
        .file_name() // Get the directory name "subtensor"
        .and_then(|os_str| os_str.to_str()) // Convert OsStr to &str
        .map(|s| s.to_string()) // Convert &str to String
}

/// Strips the longest common prefix from two paths (i.e. base is allowed to have more
/// components that are not shared with target and these are ignored)
pub fn strip_common_prefix<'a>(base: &'a Path, target: &'a Path) -> &'a Path {
    let mut base_components = base.components();
    let mut target_components = target.components();
    let mut common_length = 0;

    // Find the longest common prefix
    while let (Some(bc), Some(tc)) = (base_components.next(), target_components.next()) {
        if bc == tc {
            common_length += 1;
        } else {
            break;
        }
    }

    // Create a Path that skips the common prefix
    let mut remaining = target;
    for _ in 0..common_length {
        remaining = remaining
            .strip_prefix(remaining.components().next().unwrap())
            .unwrap_or(remaining);
    }

    remaining
}

/// Strips the longest common suffix from two paths (i.e. base is allowed to have more
/// leading components that are not shared with target and these are ignored)
pub fn strip_common_suffix<'a>(base: &'a Path, target: &'a Path) -> &'a Path {
    let base_components: Vec<_> = base.components().collect();
    let target_components: Vec<_> = target.components().collect();

    let mut common_suffix_length = 0;

    // Reverse iterate over both paths to find the longest common suffix
    for (bc, tc) in base_components
        .iter()
        .rev()
        .zip(target_components.iter().rev())
    {
        if bc == tc {
            common_suffix_length += 1;
        } else {
            break;
        }
    }

    // If there is no common suffix, return target verbatim
    if common_suffix_length == 0 {
        return target;
    }

    // Create a new path without the common suffix
    let mut remaining = target;

    for _ in 0..common_suffix_length {
        remaining = remaining.parent().unwrap_or(target);
    }

    remaining
}
