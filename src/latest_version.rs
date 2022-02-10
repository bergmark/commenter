use std::process::Command;

use crate::prelude::*;
use crate::types::*;

pub fn latest_version(packages: impl Iterator<Item = Package>) -> BTreeMap<Package, Version> {
    String::from_utf8(
        Command::new("latest-version")
            .args(packages.map(|p| p.0))
            .output()
            .expect("Could not find latest-version in PATH")
            .stdout,
    )
    .unwrap()
    .trim()
    .to_owned()
    .lines()
    .map(|s| {
        let VersionedPackage { package, version } = parse_versioned_package_canonical(s).unwrap();
        (package, version)
    })
    .collect()
}

fn parse_versioned_package_canonical(s: &str) -> Option<VersionedPackage> {
    if let Some(caps) = regex!(r#"^(.+)-([\d.]+)$"#).captures(s) {
        let package = Package(caps.get(1).unwrap().as_str().to_owned());
        let version = Version(caps.get(2).unwrap().as_str().to_owned());
        Some(VersionedPackage { package, version })
    } else {
        None
    }
}
