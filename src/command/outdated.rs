use crate::prelude::*;

use crate::handle::{handle, DisabledPackage};
use crate::latest_version::latest_version;
use crate::types::*;

pub fn outdated(build_constraints: &Path) {
    let mut all: Vec<String> = vec![];
    let (versioned, disabled) = handle(build_constraints, false, |_loc, lines| {
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
        "ghci",
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
