fn main() {
    println!("cargo:rerun-if-changed=src");
    subtensor_linting::walk_src();

    #[cfg(feature = "code-coverage")]
    {
        use std::env;
        use std::path::Path;
        let source_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let source_dir = Path::new(&source_dir);
        let rust_files = subtensor_code_coverage::collect_rust_files(source_dir);
        // Generate code coverage report
        subtensor_code_coverage::analyze_files(&rust_files, source_dir);
    }
}
