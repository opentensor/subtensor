fn main() {
    // Force the `std` feature in some cases
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    if target_os == "wasi" || target_os == "emscripten" || target_os == "hermit" {
        println!("cargo:rustc-cfg=feature=\"std\"");
    }
}
