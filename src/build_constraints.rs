use crate::prelude::*;

use std::fmt;

use crate::regex::cap_into_opt;
use crate::types::{Package, Version};
use lazy_regex::regex;
use serde::{Deserialize, Deserializer};

#[derive(Deserialize, Debug, PartialEq, Eq)]
struct BuildConstraints {
    #[serde(rename = "ghc-version")]
    ghc_version: String,
    packages: BTreeMap<String, Vec<BCPackage>>,
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Debug, Clone)]
pub struct Maintainer(pub String);

impl fmt::Display for Maintainer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Maintainer {
    pub fn github_users(&self) -> impl Iterator<Item = &str> {
        regex!(" +")
            .split(&self.0)
            .filter(|s| regex!("^@[^ ]+$").is_match(s))
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct BCPackage {
    pub package: Package,
    pub bound: Option<String>,
    pub version: Option<Version>,
}

impl<'de> serde::Deserialize<'de> for BCPackage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = serde::Deserialize::deserialize(deserializer)?;
        let r = regex!(
            r#"^(?P<package>[\da-zA-z][\da-zA-Z-]*) *(?:(?P<bound>[<>=^]+) *(?P<version>(\d+(?:\.\d+)*)))? *(?:#.+)?"#
        );
        let cap = &r.captures(&s).unwrap();
        let package = cap_into_opt(cap, "package").unwrap();
        let bound = cap_into_opt(cap, "bound");
        let version = cap_into_opt(cap, "version");
        Ok(BCPackage {
            package,
            bound,
            version,
        })
    }
}

pub fn transpose(m: BTreeMap<Maintainer, Vec<BCPackage>>) -> BTreeMap<Package, Vec<Maintainer>> {
    let mut res = BTreeMap::new();
    for (maintainer, packages) in m {
        for BCPackage { package, .. } in packages {
            let e = res.entry(package).or_insert_with(Vec::new);
            e.push(maintainer.clone());
        }
    }
    res
}

pub struct ParsedBuildConstraints {
    pub ghc_version: String,
    pub packages: BTreeMap<Maintainer, Vec<BCPackage>>,
}

impl ParsedBuildConstraints {
    pub fn maintainers(&self) -> impl Iterator<Item = &Maintainer> {
        self.packages.keys()
    }
}

pub fn parse(f: &Path) -> ParsedBuildConstraints {
    use crate::yaml;
    let BuildConstraints {
        ghc_version,
        packages,
    } = yaml::yaml_from_file(f)
        .unwrap_or_else(|e| panic!("Could not open build-constraints file at {f:?}, error: {e}"));
    let packages = packages
        .into_iter()
        .filter_map(|(k, v)| {
            if [
                "Grandfathered dependencies",
                "Abandoned packages",
                "Unmaintained packages with compilation failures",
                "Removed packages",
                "GHC upper bounds",
                "Compilation failures",
                "Library and exe bounds failures",
                "Stackage upper bounds",
            ]
            .contains(&&*k)
            {
                None
            } else {
                Some((Maintainer(k), v))
            }
        })
        .collect();
    ParsedBuildConstraints {
        ghc_version,
        packages,
    }
}

#[test]
fn test_parse_build_constraints() {
    let _ = parse(&PathBuf::from("test/build-constraints.yaml"));
}
