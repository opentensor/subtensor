use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

fn main() {
    // Get the root directory of the workspace
    let workspace_root = env::var("CARGO_MANIFEST_DIR").unwrap();
    let workspace_root = Path::new(&workspace_root);

    // Collect all Rust source files in the workspace
    let rust_files = collect_rust_files(workspace_root);

    // Parse each Rust file with syn
    for file in rust_files {
        let Ok(content) = fs::read_to_string(&file) else {
            continue;
        };
        let Ok(parsed_file) = syn::parse_file(&content) else {
            continue;
        };
        //println!("{}", parsed_file.items.len())
    }
}

// Recursively collects all Rust files in the given directory
fn collect_rust_files(dir: &Path) -> Vec<PathBuf> {
    let mut rust_files = Vec::new();

    for entry in WalkDir::new(dir) {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            rust_files.push(path.to_path_buf());
        }
    }

    rust_files
}
