use std::env;
use std::path::PathBuf;

fn main() {
    // should we instead provide an argument to set the directory?
    let source_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR should be set");
    let mut source_dir = PathBuf::from(&source_dir);
    source_dir.pop();
    source_dir.pop();
    let rust_files = subtensor_code_coverage::collect_rust_files(source_dir.as_path());

    // Generate code coverage report
    subtensor_code_coverage::analyze_files(&rust_files, source_dir.as_path());
}
