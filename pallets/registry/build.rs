fn main() {
    println!("cargo:rerun-if-changed=src");
    subtensor_linting::walk_src();
}
