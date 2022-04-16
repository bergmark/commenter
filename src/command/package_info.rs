use crate::prelude::*;

use crate::build_constraints::{self, BCPackage2};
use crate::latest_version;
use crate::snapshot::{self, FoundSnapshots, Lts, Nightly, SnapshotYaml};
use crate::types::{Package, Version};

use itertools::Itertools;

pub fn package_info(
    stackage_snapshots_path: &Path,
    no_search_snapshots: bool,
    build_constraints: &Path,
    package: &str,
) {
    let package = Package(package.to_owned());
    println!("{package}:");

    match latest_version::latest_version_for(&package) {
        Some(version) => eprintln!("Hackage: latest version: {version}"),
        None => eprintln!("Hackage: Could not find package"),
    }

    let bc = build_constraints::parse(build_constraints).by_package();

    if let Some(BCPackage2 {
        bounds,
        maintainers,
    }) = bc.package(&package)
    {
        if bounds.is_empty() {
            println!("build-constraints: bounds: None");
        } else {
            println!(
                "build-constraints: bounds: {bounds}",
                bounds = bounds.iter().map(|b| b.to_string()).join(", ")
            );
        }
        if maintainers.is_empty() {
            println!("build-constraints: Not present in maintainer sections!");
        } else {
            println!(
                "build-constraints: maintainer sections: {maintainers}",
                maintainers = maintainers.iter().map(|m| m.to_string()).join(", ")
            );
        }
    } else {
        println!("build-constraints: Could not find package");
    }

    if !no_search_snapshots {
        match find_latest_snapshots_with_package(stackage_snapshots_path, &package) {
            Err(e) => eprintln!("ERROR querying stackage-snapshots: {e}"),
            Ok(Res { nightly, lts }) => {
                match nightly {
                    None => eprintln!("Could not find package in nightly"),
                    Some((snapshot, version)) => {
                        println!("nightly: Latest snapshot: {snapshot}");
                        println!("nightly: latest version: {version}");
                    }
                };
                match lts {
                    None => eprintln!("Could not find package in LTS"),
                    Some((snapshot, version)) => {
                        println!("LTS: latest snapshot {snapshot}");
                        println!("LTS: latest version: {version}");
                    }
                };
            }
        }
    }
}

struct Res {
    nightly: Option<(Nightly, Version)>,
    lts: Option<(Lts, Version)>,
}

fn find_latest_snapshots_with_package(
    stackage_snapshots_path: &Path,
    package: &Package,
) -> Result<Res, anyhow::Error> {
    let mut latest_nightly = None;
    let mut latest_lts = None;
    let FoundSnapshots {
        mut nightly,
        mut lts,
    } = snapshot::find_snapshots(stackage_snapshots_path).context("ERROR finding snapshots")?;
    nightly.sort();
    for (nightly, path) in nightly.into_iter().rev() {
        let s: SnapshotYaml = crate::yaml::yaml_from_file(&path)
            .with_context(|| format!("ERROR parsing snapshot {path}", path = path.display()))?;
        if let Some(sp) = s.packages.iter().find(|p| &p.hackage.0.package == package) {
            latest_nightly = Some((nightly, sp.hackage.0.version.clone()));
            break;
        }
    }
    lts.sort();
    for (lts, path) in lts.into_iter().rev() {
        let s: SnapshotYaml = crate::yaml::yaml_from_file(&path)
            .with_context(|| format!("ERROR parsing snapshot {path}", path = path.display()))?;
        if let Some(sp) = s.packages.iter().find(|p| &p.hackage.0.package == package) {
            latest_lts = Some((lts, sp.hackage.0.version.clone()));
            break;
        }
    }
    Ok(Res {
        nightly: latest_nightly,
        lts: latest_lts,
    })
}
