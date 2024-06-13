use std::env;

fn main() {
    // Read the RUSTC_WRAPPER environment variable
    if let Ok(rustc_wrapper) = env::var("RUSTC_WRAPPER") {
        // Pass the RUSTC_WRAPPER to rust build
        println!("cargo:rustc-env=RUSTC_WRAPPER={}", rustc_wrapper);
    }
}
