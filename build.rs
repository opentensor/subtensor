use rayon::prelude::*;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use syn::Result;
use walkdir::WalkDir;

mod lints;
use lints::*;

fn main() {
    // Get the root directory of the workspace
    let workspace_root = env::var("CARGO_MANIFEST_DIR").unwrap();
    let workspace_root = Path::new(&workspace_root);

    // Collect all Rust source files in the workspace
    let rust_files = collect_rust_files(workspace_root);

    // Channel used to communicate errors back to the main thread from the parallel processing
    // as we process each Rust file
    let (tx, rx) = channel();

    // Parse each rust file with syn and run the linting suite on it in parallel
    rust_files.par_iter().for_each_with(tx.clone(), |tx, file| {
        let Ok(content) = fs::read_to_string(&file) else {
            return;
        };
        let Ok(parsed_file) = syn::parse_file(&content) else {
            return;
        };

        let track_lint = |result: Result<()>| {
            let Err(error) = result else {
                return;
            };
            tx.send((error, file.clone())).unwrap();
        };

        track_lint(DummyLint::lint(&parsed_file));
        track_lint(RequireFreezeStruct::lint(&parsed_file));
    });

    // Collect and print all errors after the parallel processing is done
    drop(tx); // Close the sending end of the channel

    for (error, file) in rx {
        let relative_path = file.strip_prefix(workspace_root).unwrap_or(file.as_path());
        let start = error.span().start();
        let end = error.span().end();
        let start_line = start.line;
        let start_col = start.column;
        let _end_line = end.line;
        let _end_col = end.column;
        let file_path = relative_path.display();
        println!(
            "cargo:warning={}:{}:{}: {}",
            file_path, start_line, start_col, error
        );
    }
}

/// Recursively collects all Rust files in the given directory
fn collect_rust_files(dir: &Path) -> Vec<PathBuf> {
    let mut rust_files = Vec::new();

    for entry in WalkDir::new(dir) {
        let entry = entry.unwrap();
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
