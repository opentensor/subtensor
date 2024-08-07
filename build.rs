use std::env;
use std::fs;
use std::path::{Path, PathBuf};
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

    let mut found_error = None;

    // Parse each Rust file with syn
    for file in rust_files {
        if found_error.is_some() {
            break;
        }
        let Ok(content) = fs::read_to_string(&file) else {
            continue;
        };
        let Ok(parsed_file) = syn::parse_file(&content) else {
            continue;
        };

        let track_lint = |result: Result<()>| {
            let Err(error) = result else {
                return;
            };
            found_error = Some((error, file));
        };

        track_lint(DummyLint::lint(parsed_file));
    }

    if let Some((error, file)) = found_error {
        let start = error.span().start();
        let end = error.span().end();
        let start_line = start.line;
        let start_col = start.column;
        let _end_line = end.line;
        let _end_col = end.column;
        let file_path = file.display();
        panic!("{}:{}:{}: {}", file_path, start_line, start_col, error);
    }
}

// Recursively collects all Rust files in the given directory
fn collect_rust_files(dir: &Path) -> Vec<PathBuf> {
    let mut rust_files = Vec::new();

    for entry in WalkDir::new(dir) {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.ends_with("target") || path.ends_with("build.rs") {
            continue;
        }

        if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            rust_files.push(path.to_path_buf());
        }
    }

    rust_files
}
