use serde::{Deserialize, Deserializer};

use crate::prelude::*;
use crate::regex::*;
use crate::types::{Package, Version, VersionedPackage};

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
            let package = cap_into_n(&caps, 1);
            let version = cap_try_into_n(&caps, 2);
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

#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct Nightly {
    pub year: usize,
    pub month: usize,
    pub day: usize,
}

impl fmt::Display for Nightly {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { year, month, day } = self;
        write!(f, "nightly-{year}-{month:02}-{day:02}")
    }
}

#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct Lts {
    pub major: usize,
    pub minor: usize,
}

impl fmt::Display for Lts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { major, minor } = self;
        write!(f, "lts-{major}.{minor}")
    }
}

pub struct FoundSnapshots {
    pub nightly: Vec<(Nightly, PathBuf)>,
    pub lts: Vec<(Lts, PathBuf)>,
}

pub fn find_snapshots(stackage_snapshots_path: &Path) -> Result<FoundSnapshots, anyhow::Error> {
    let mut nightly: Vec<(Nightly, PathBuf)> = vec![];
    let mut lts: Vec<(Lts, PathBuf)> = vec![];
    let mut dirs: BTreeSet<PathBuf> = BTreeSet::from([stackage_snapshots_path.into()]);
    let mut dirs_len = dirs.len();
    while dirs_len >= 1 {
        let mut new_dirs = BTreeSet::new();
        for current_dir in dirs {
            if current_dir.ends_with(".git") {
                continue;
            }

            for entry in fs::read_dir(current_dir).unwrap() {
                let entry = entry.unwrap();
                let path = entry.path();
                let metadata = fs::metadata(&path).unwrap();
                let file_type = metadata.file_type();

                if file_type.is_dir() {
                    new_dirs.insert(path);
                } else if path.extension() == Some(OsStr::new("yaml")) {
                    let path_str = path.to_str().unwrap();
                    if let Some(caps) =
                        regex!(r#"/nightly/(\d+)/(\d+)/(\d+).yaml$"#).captures(path_str)
                    {
                        let n = Nightly {
                            year: caps[1].parse().unwrap(),
                            month: caps[2].parse().unwrap(),
                            day: caps[3].parse().unwrap(),
                        };
                        nightly.push((n, path));
                    } else if let Some(caps) =
                        regex!(r#"/lts/(\d+)/(\d+).yaml$"#).captures(path_str)
                    {
                        let n = Lts {
                            major: caps[1].parse().unwrap(),
                            minor: caps[2].parse().unwrap(),
                        };
                        lts.push((n, path));
                    }
                }
            }
        }
        dirs = new_dirs;
        dirs_len = dirs.len();
    }

    Ok(FoundSnapshots { nightly, lts })
}
