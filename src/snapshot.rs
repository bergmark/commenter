use serde::{Deserialize, Deserializer};

use crate::prelude::*;
use crate::types::{Package, Version, VersionedPackage};
use crate::yaml;

#[derive(Deserialize)]
pub struct SnapshotYaml {
    // flags: BTreeMap<Package, BTreeMap<PackageFlag, bool>>,
    // publish_time
    pub packages: Vec<SnapshotPackage>,
    // hidden
    // resolver
}

#[derive(Deserialize)]
pub struct SnapshotPackage {
    pub hackage: PackageWithVersionAndSha,
    // pantry-tree
}

// zstd-0.1.3.0@sha256:4c0a372251068eb6086b8c3a0a9f347488f08b570a7705844ffeb2c720c97223,3723
pub struct PackageWithVersionAndSha(pub VersionedPackage);

impl<'de> serde::Deserialize<'de> for PackageWithVersionAndSha {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = String::deserialize(deserializer)?;
        let r = regex!(r#"^(.+?)-([.\d]+)@sha256:[\da-z]+,\d+$"#);
        if let Some(caps) = r.captures(&s) {
            let package = Package(caps.get(1).unwrap().as_str().to_owned());
            let version = Version(caps.get(2).unwrap().as_str().to_owned());
            Ok(Self(VersionedPackage { package, version }))
        } else {
            Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Other(&s),
                &"Invalid PackageVersionWithSha",
            ))
        }
    }
}

pub struct Snapshot {
    pub packages: BTreeMap<Package, Diff<Version>>,
}

#[derive(Clone, Copy)]
pub enum Diff<A> {
    Left(A),
    Right(A),
    Both(A, A),
}

pub fn to_diff(a: SnapshotYaml, b: SnapshotYaml) -> Snapshot {
    let mut packages = BTreeMap::new();
    for s in a.packages {
        let package = s.hackage.0;
        packages.insert(package.package, Diff::Left(package.version));
    }
    for s in b.packages {
        let package = s.hackage.0;
        let name = package.package;
        let version = package.version;
        if let Some(a) = packages.remove(&name) {
            match a {
                Diff::Left(a) => {
                    if a == version {
                        packages.remove(&name);
                    } else {
                        packages.insert(name, Diff::Both(a, version));
                    }
                }
                _ => unreachable!(),
            }
        } else {
            packages.insert(name, Diff::Right(version));
        }
    }

    Snapshot { packages }
}

pub fn diff_snapshot(a: String, b: String) {
    let diff = to_diff(
        yaml::yaml_from_file(a).unwrap(),
        yaml::yaml_from_file(b).unwrap(),
    );
    for (name, diff) in diff.packages {
        let s = match diff {
            Diff::Left(a) => format!("- {name}-{a}"),
            Diff::Right(b) => format!("+ {name}-{b}"),
            Diff::Both(a, b) => format!("^ {name}-{a} -> {b}"),
        };
        println!("{s}");
    }
}
