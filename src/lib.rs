use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::{BufRead, BufReader, LineWriter, Lines, Write};
use std::path::Path;
use std::process::Command;

pub mod add;
mod prelude;
mod regex;
pub mod snapshot;
mod types;

use prelude::*;
use types::*;

pub fn clear() {
    handle(true, |loc, _lines| match loc {
        // Add empty array to keep yaml valid
        Location::Lib => vec!["        []".to_owned()],
        Location::Test | Location::Bench => vec![],
    });
}

pub fn add(lib: Vec<String>, test: Vec<String>, bench: Vec<String>) {
    handle(true, |loc, mut lines| {
        lines.extend(match loc {
            Location::Lib => lib.clone(),
            Location::Test => test.clone(),
            Location::Bench => bench.clone(),
        });
        lines.sort();
        lines
    });
}

enum VersionTag {
    Manual(Version),
    Auto(Version),
}

impl VersionTag {
    fn tag(&self) -> &'static str {
        match self {
            VersionTag::Manual(_) => "manual",
            VersionTag::Auto(_) => "auto",
        }
    }

    fn version(&self) -> &Version {
        match self {
            VersionTag::Manual(s) => s,
            VersionTag::Auto(s) => s,
        }
    }
}

pub fn outdated() {
    let mut all: Vec<String> = vec![];
    let (versioned, disabled) = handle(false, |_loc, lines| {
        all.extend(lines);
        vec![]
    });

    for DisabledPackage { package } in disabled {
        println!("WARN: {package} is disabled without a noted version");
    }

    let mut map: BTreeMap<Package, VersionTag> = BTreeMap::new();
    for VersionedPackage { package, version } in versioned {
        map.insert(package, VersionTag::Manual(version));
    }
    let mut support: BTreeMap<(Package, Version), BTreeSet<(Package, Version)>> = BTreeMap::new();
    for v in all.into_iter() {
        let caps = regex!("tried ([^ ]+)-([^,-]+),").captures(&v).unwrap();
        let package = Package(caps.get(1).unwrap().as_str().to_owned());
        let version = Version(caps.get(2).unwrap().as_str().to_owned());
        map.insert(package.clone(), VersionTag::Auto(version.clone()));

        if let Some(caps) = regex!("does not support: ([^ ]+)-([^-]+)").captures(&v) {
            let dep_package = Package(caps.get(1).unwrap().as_str().to_owned());
            let dep_version = Version(caps.get(2).unwrap().as_str().to_owned());
            let entry = support.entry((dep_package, dep_version)).or_default();
            entry.insert((package, version));
        }
    }

    let latest_versions = {
        let mut packages: Vec<Package> = map.iter().map(|(package, _)| package.clone()).collect();
        packages.append(
            &mut support
                .iter()
                .map(|((package, _), _)| package.clone())
                .collect(),
        );
        latest_version(packages.into_iter())
    };

    for (package, version) in map {
        if is_boot(&package) {
            continue;
        }
        let latest = latest_versions.get(&package).unwrap();
        if version.version() != latest {
            println!(
                "{package} mismatch, {tag}: {version}, hackage: {latest}",
                tag = version.tag(),
                version = version.version(),
            );
        }
    }

    for ((package, version), dependents) in support {
        if is_boot(&package) {
            continue;
        }

        let latest = latest_versions.get(&package).unwrap();
        if &version != latest {
            let max = 3;
            let dependents_stripped = dependents.len().saturating_sub(max);
            let dependents = dependents
                .into_iter()
                .take(max)
                .map(|(p, v)| format!("{p}-{v}"))
                .collect::<Vec<String>>()
                .join(", ");
            let dependents = if dependents_stripped > 0 {
                format!("{dependents} and {dependents_stripped} more")
            } else {
                dependents
            };

            println!(
                "{package} mismatch, snapshot: {version}, hackage: {latest}, dependents: {dependents}"
            );
        }
    }
}

fn is_boot(package: &Package) -> bool {
    [
        "Cabal",
        "base",
        "bytestring",
        "containers",
        "containers",
        "directory",
        "filepath",
        "deepseq",
        "ghc",
        "ghc-bignum",
        "ghc-boot",
        "ghc-boot-th",
        "ghc-prim",
        "ghc-lib-parser", // not a boot lib, but tied to the GHC version.
        "integer-gmp",
        "process",
        "stm",
        "template-haskell",
        "text",
        "time",
    ]
    .contains(&&*package.0)
}

fn latest_version(packages: impl Iterator<Item = Package>) -> BTreeMap<Package, Version> {
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

enum State {
    LookingForLibBounds,
    ProcessingLibBounds,
    LookingForTestBounds,
    ProcessingTestBounds,
    LookingForBenchBounds,
    ProcessingBenchBounds,
    Done,
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

fn parse_versioned_package_yaml(s: &str) -> Option<VersionedPackage> {
    if let Some(caps) = regex!(r#"- *([^ ]+) < *0 *# *([\d.]+)"#).captures(s) {
        let package = Package(caps.get(1).unwrap().as_str().to_owned());
        let version = Version(caps.get(2).unwrap().as_str().to_owned());
        Some(VersionedPackage { package, version })
    } else if let Some(caps) = regex!(r#"- *([^ ]+) *# *([\d.]+)"#).captures(s) {
        let package = Package(caps.get(1).unwrap().as_str().to_owned());
        let version = Version(caps.get(2).unwrap().as_str().to_owned());
        Some(VersionedPackage { package, version })
    } else {
        None
    }
}

struct DisabledPackage {
    package: String,
}

fn parse_disabled_package(s: &str) -> Option<DisabledPackage> {
    if !regex!(r#"- *([^ ]+) < *0 *# tried"#).is_match(s) {
        if let Some(caps) = regex!(r#"- *([^ ]+) < *0 *# *\d*[^\d ]"#).captures(s) {
            let package = caps.get(1).unwrap().as_str().to_owned();
            Some(DisabledPackage { package })
        } else {
            None
        }
    } else {
        None
    }
}

fn handle<F>(write: bool, mut f: F) -> (Vec<VersionedPackage>, Vec<DisabledPackage>)
where
    F: FnMut(Location, Vec<String>) -> Vec<String>,
{
    let path = "build-constraints.yaml";
    let mut new_lines: Vec<String> = vec![];
    let mut versioned_packages: Vec<VersionedPackage> = vec![];
    let mut disabled_packages: Vec<DisabledPackage> = vec![];

    let mut state = State::LookingForLibBounds;
    let mut buf = vec![];
    for line in read_lines(path).map(|s| s.unwrap()) {
        if let Some(versioned_package) = parse_versioned_package_yaml(&line) {
            versioned_packages.push(versioned_package);
        } else if let Some(disabled_package) = parse_disabled_package(&line) {
            disabled_packages.push(disabled_package);
        }

        match state {
            State::LookingForLibBounds => {
                if line == r#"    "Library and exe bounds failures":"# {
                    state = State::ProcessingLibBounds;
                }
                new_lines.push(line);
            }
            State::ProcessingLibBounds => {
                if line == r#"    # End of Library and exe bounds failures"# {
                    new_lines.extend(f(Location::Lib, buf).into_iter());
                    buf = vec![];
                    new_lines.push(line);
                    state = State::LookingForTestBounds;
                } else {
                    // Remove empty section
                    if line != "        []" {
                        buf.push(line);
                    }
                }
            }
            State::LookingForTestBounds => {
                if line == r#"    # Test bounds issues"# {
                    state = State::ProcessingTestBounds;
                }
                new_lines.push(line);
            }
            State::ProcessingTestBounds => {
                if line == r#"    # End of Test bounds issues"# {
                    new_lines.extend(f(Location::Test, buf).into_iter());
                    buf = vec![];
                    new_lines.push(line);
                    state = State::LookingForBenchBounds;
                } else {
                    buf.push(line);
                }
            }
            State::LookingForBenchBounds => {
                if line == r#"    # Benchmark bounds issues"# {
                    state = State::ProcessingBenchBounds;
                }
                new_lines.push(line);
            }
            State::ProcessingBenchBounds => {
                if line == r#"    # End of Benchmark bounds issues"# {
                    new_lines.extend(f(Location::Bench, buf).into_iter());
                    buf = vec![];
                    new_lines.push(line);
                    state = State::Done;
                } else {
                    buf.push(line);
                }
            }
            State::Done => {
                new_lines.push(line);
            }
        }
    }

    if write {
        let file = File::create(path).unwrap();
        let mut file = LineWriter::new(file);

        for line in new_lines {
            file.write_all((line + "\n").as_bytes()).unwrap();
        }
        file.flush().unwrap();
    }

    (versioned_packages, disabled_packages)
}

enum Location {
    Lib,
    Test,
    Bench,
}

fn read_lines<P>(filename: P) -> Lines<BufReader<File>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename).unwrap();
    BufReader::new(file).lines()
}

#[derive(PartialEq, Eq, Debug)]
struct DisabledTransitively {
    child: VersionedPackage,
    parent: Package,
}

fn parse_disabled_transitviely(s: &str) -> Option<DisabledTransitively> {
    let r = regex!(
        r#"- *([^ ]+) < *0 *# tried [^ ]+-([\d.]+), but its \*[^*]+\* requires the disabled package: ([^ ]+)"#
    );
    if let Some(caps) = r.captures(s) {
        let package = Package(caps.get(1).unwrap().as_str().to_owned());
        let version = Version(caps.get(2).unwrap().as_str().to_owned());
        let parent = Package(caps.get(3).unwrap().as_str().to_owned());
        Some(DisabledTransitively {
            child: VersionedPackage { package, version },
            parent,
        })
    } else {
        None
    }
}

#[test]
fn test_parse_disabled_transitviely() {
    let s = "- Network-NineP < 0 # tried Network-NineP-0.4.7.1, but its *library* requires the disabled package: mstate";
    assert_eq!(
        parse_disabled_transitviely(s),
        Some(DisabledTransitively {
            child: VersionedPackage {
                package: Package("Network-NineP".to_owned()),
                version: Version("0.4.7.1".to_owned())
            },
            parent: Package("mstate".to_owned()),
        })
    )
}

type M = BTreeMap<Package, (Vec<VersionedPackage>, Option<usize>)>;

pub fn disabled() {
    let mut disabled_transitively: Vec<DisabledTransitively> = vec![];
    handle(false, |loc, lines| {
        match loc {
            Location::Lib => disabled_transitively.extend(
                lines
                    .into_iter()
                    .map(|line| parse_disabled_transitviely(&line))
                    .flatten()
                    .collect::<Vec<_>>(),
            ),
            Location::Test | Location::Bench => (),
        }
        vec![]
    });

    let mut packages: BTreeSet<Package> = BTreeSet::new();
    let mut disabled: M = BTreeMap::new();

    for DisabledTransitively { child, parent } in disabled_transitively {
        packages.insert(child.package.clone());
        packages.insert(parent.clone());
        disabled
            .entry(child.package.clone())
            .or_insert_with(|| (vec![], None));
        let t = disabled.entry(parent).or_insert_with(|| (vec![], None));
        t.0.push(child);
    }

    let mut packages_len = packages.len();
    while packages_len > 0 {
        let mut new_packages: BTreeSet<Package> = BTreeSet::new();
        for package in packages {
            let (_, count) = disabled.get(&package).unwrap();
            if count.is_none() && !process(&package, &mut disabled) {
                new_packages.insert(package.clone());
            }
        }
        packages = new_packages;
        packages_len = packages.len();
    }

    let mut v: Vec<_> = disabled
        .into_iter()
        .map(|(package, (_, count))| (count, package))
        .collect();
    v.sort();
    for (count, package) in v {
        let count = count.unwrap();
        if count != 0 {
            println!("{package} is disabled with {count} dependents");
        }
    }
}

fn process(package: &Package, m: &mut M) -> bool {
    let (children, count) = m.get(package).unwrap();
    assert!(count.is_none(), "{:?}", package);
    let mut count = 0;
    for child in children {
        let (_, child_count) = m
            .get(&child.package)
            .unwrap_or_else(|| panic!("{}", child.package));
        match child_count {
            None => return false,
            Some(i) => count += 1 + i,
        }
    }
    m.entry(package.clone())
        .and_modify(|tup| tup.1 = Some(count))
        .or_insert_with(|| panic!("{}", package));
    true
}
