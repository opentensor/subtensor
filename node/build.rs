use substrate_build_script_utils::{generate_cargo_keys, rerun_if_git_head_changed};

fn main() {
    generate_cargo_keys();
    rerun_if_git_head_changed();

    println!("cargo:rerun-if-changed=src");
    subtensor_linting::walk_src();
}
