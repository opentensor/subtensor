#!/usr/bin/env rust-script
// ^ `cargo install rust-script` to be able to run this script

use core::fmt::Display;
use std::{env, process::Command};

fn eval(cmd: impl Display) -> String {
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd.to_string())
        .output()
        .expect("failed to execute process");
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

fn main() {
    let previous_tag = env::var("PREVIOUS_TAG").unwrap_or_else(|_| {
        eval("git describe --abbrev=0 --tags $(git rev-list --tags --skip=1 --max-count=1)")
    });
    if previous_tag.is_empty() {
        panic!("PREVIOUS_TAG is not specified or invalid");
    }
    println!("Previous tag: {}", previous_tag);
}
