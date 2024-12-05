use std::env;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::{fs, str::FromStr};

use proc_macro2::TokenTree;
use syn::{Attribute, File, Meta, MetaList, Path as SynPath};

use rayon::prelude::*;
use walkdir::WalkDir;

use crate::{
    DisallowV1Benchmarks, ForbidAsPrimitiveConversion, RequireExplicitPalletIndex,
    RequireFreezeStruct,
};

pub type Result = core::result::Result<(), Vec<syn::Error>>;

/// A trait that defines custom lints that can be run within our workspace.
///
/// Each lint is run in parallel on all Rust source files in the workspace. Within a lint you
/// can issue an error the same way you would in a proc macro, and otherwise return `Ok(())` if
/// there are no errors.
pub trait Lint: Send + Sync {
    /// Lints the given Rust source file, returning a compile error if any issues are found.
    fn lint(source: &File) -> Result;
}

pub fn is_allowed(attibutes: &[Attribute]) -> bool {
    attibutes.iter().any(|attribute| {
        let Attribute {
            meta:
                Meta::List(MetaList {
                    path: SynPath { segments: attr, .. },
                    tokens: attr_args,
                    ..
                }),
            ..
        } = attribute
        else {
            return false;
        };

        attr.len() == 1
            && attr[0].ident == "allow"
            && attr_args.clone().into_iter().any(|arg| {
                let TokenTree::Ident(ref id) = arg else {
                    return false;
                };
                id == "unknown_lints"
            })
    })
}

pub fn walk_src() {
    let source_dir =
        env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is expected to be set");
    let source_dir = Path::new(&source_dir);

    let rust_files = collect_rust_files(source_dir);

    let (tx, rx) = mpsc::channel();

    // Parse each rust file with syn and run the linting suite on it in parallel
    rust_files.par_iter().for_each_with(tx.clone(), |tx, file| {
        let Ok(content) = fs::read_to_string(file) else {
            return;
        };
        let Ok(parsed_tokens) = proc_macro2::TokenStream::from_str(&content) else {
            return;
        };
        let Ok(parsed_file) = syn::parse2::<syn::File>(parsed_tokens) else {
            return;
        };

        let track_lint = |result: Result| {
            let Err(errors) = result else {
                return;
            };
            let relative_path = file.strip_prefix(source_dir).unwrap_or(file.as_path());
            for error in errors {
                let loc = error.span().start();
                let file_path = relative_path.display();
                // note that spans can't go across thread boundaries so we serialize here and send a String
                tx.send(format!(
                    "cargo:warning={}:{}:{}: {}",
                    file_path, loc.line, loc.column, error,
                ))
                .expect("sending via unbound channel should work");
            }
        };

        track_lint(DisallowV1Benchmarks::lint(&parsed_file));
        track_lint(ForbidAsPrimitiveConversion::lint(&parsed_file));
        track_lint(RequireFreezeStruct::lint(&parsed_file));
        track_lint(RequireExplicitPalletIndex::lint(&parsed_file));
    });

    // Collect and print all errors after the parallel processing is done
    drop(tx); // Close the sending end of the channel

    for error in rx {
        println!("{error}");
    }
}

// copied from the `code-coverage` crate, to not introduce dependency
fn collect_rust_files(dir: &Path) -> Vec<PathBuf> {
    let mut rust_files = Vec::new();

    for entry in WalkDir::new(dir) {
        let Ok(entry) = entry else {
            continue;
        };
        let path = entry.path();

        // Skip any path that contains "target" directory
        if path
            .components()
            .any(|component| component.as_os_str() == "target")
            || path.ends_with("build.rs")
        {
            continue;
        }

        if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            rust_files.push(path.to_path_buf());
        }
    }

    rust_files
}
