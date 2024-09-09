#!/usr/bin/env rust-script
// ^ `cargo install rust-script` to be able to run this script

use core::{fmt::Display, str::FromStr};
use std::{env, process::Command};

fn eval(cmd: impl Display, print: bool) -> Result<String, String> {
    if print {
        println!("$ {}", cmd);
    }
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd.to_string())
        .output()
        .expect("failed to execute process");
    if print {
        println!("{}", String::from_utf8(output.stdout.clone()).unwrap());
        eprintln!("{}", String::from_utf8(output.stderr.clone()).unwrap());
    }
    if !output.status.success() {
        return Err(String::from_utf8(output.stderr).unwrap());
    }
    Ok(String::from_utf8(output.stdout).unwrap().trim().to_string())
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Network {
    Mainnet,
    Testnet,
}

impl FromStr for Network {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "mainnet" => Ok(Network::Mainnet),
            "testnet" => Ok(Network::Testnet),
            _ => Err(()),
        }
    }
}

fn main() {
    let network = env::var("NETWORK")
        .unwrap_or_else(|_| "mainnet".to_string())
        .parse::<Network>()
        .unwrap_or_else(|_| panic!("Invalid NETWORK value"));
    println!("Network: {:?}", network);

    let all_tags = env::var("PREVIOUS_TAG")
        .unwrap_or_else(|_| eval("git tag --sort=-creatordate", false).unwrap())
        .split("\n")
        .map(|s| s.trim().to_string())
        .collect::<Vec<String>>();

    let previous_tag = match network {
        Network::Mainnet => all_tags
            .iter()
            .find(|tag| tag.starts_with("v") && !tag.ends_with("-pre-release"))
            .expect("could not find a valid mainnet tag!"),
        Network::Testnet => all_tags
            .iter()
            .find(|tag| tag.starts_with("v") && tag.ends_with("-pre-release"))
            .expect("could not find a valid testnet tag!"),
    };
    println!("Previous release tag: {}", previous_tag);

    let branch = env::var("BRANCH").unwrap_or(
        match network {
            Network::Mainnet => "testnet",
            Network::Testnet => "devnet",
        }
        .to_string(),
    );
    println!("Branch: {}", branch);

    println!(
        "Generating release notes for all merges since {}...",
        previous_tag,
    );
    let merges = eval(
        format!(
            "git log --merges --pretty=format:'%s' {}..{}",
            previous_tag,
            branch // Replace HEAD with branch variable
        ),
        false,
    )
    .unwrap()
    .split("\n")
    .map(|s| s.trim().to_string())
    .filter(|s| {
        !s.is_empty()
            && s.starts_with("Merge pull request #")
            && !s.ends_with("from opentensor/devnet-ready")
            && !s.ends_with("from opentensor/testnet-ready")
            && !s.ends_with("from opentensor/devnet")
            && !s.ends_with("from opentensor/testnet")
    })
    .collect::<Vec<String>>();

    println!("");
    println!("Filtered merges:\n{}", merges.join("\n"));

    println!("");
}
