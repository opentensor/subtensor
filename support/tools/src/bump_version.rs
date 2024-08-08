use semver::Version;
use std::{
    fs,
    io::{Read, Seek, Write},
    str::FromStr,
};
use toml_edit::{DocumentMut, Item, Value};

const TOML_PATHS: [&str; 9] = [
    "support/macros",
    "pallets/commitments",
    "pallets/collective",
    "pallets/registry",
    "pallets/subtensor",
    "pallets/subtensor/runtime-api",
    "pallets/admin-utils",
    "runtime",
    "node",
];

fn main() -> anyhow::Result<()> {
    let mut version_file = fs::File::options().read(true).write(true).open("VERSION")?;
    let mut version_str = String::new();
    version_file.read_to_string(&mut version_str)?;
    let mut version = Version::parse(&version_str)?;
    version.minor = version.minor.saturating_add(1);

    for path in TOML_PATHS {
        let cargo_toml_path = format!("{path}/Cargo.toml");
        let mut toml_file = fs::File::options()
            .read(true)
            .write(true)
            .open(&cargo_toml_path)?;
        let mut toml_str = String::new();
        toml_file.read_to_string(&mut toml_str)?;
        let mut modified_toml_doc = DocumentMut::from_str(&toml_str)?;

        modified_toml_doc["package"]["version"] = Item::Value(Value::from(version.to_string()));
        toml_file.set_len(0)?;
        toml_file.rewind()?;
        toml_file.write_all(modified_toml_doc.to_string().as_bytes())?;
    }

    version_file.set_len(0)?;
    version_file.rewind()?;
    version_file.write_all(version.to_string().as_bytes())?;

    Ok(())
}
