use crate::prelude::*;
use crate::regex::cap_into_opt;
use crate::types::{Package, Version};
use lazy_regex::regex;
use serde::{Deserialize, Deserializer};

#[derive(Deserialize, Debug, PartialEq, Eq)]
struct BuildConstraints {
    packages: BTreeMap<String, Vec<BCPackage>>,
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Debug, Clone)]
pub struct Maintainer(pub String);

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

pub fn parse(f: &str) -> BTreeMap<Maintainer, Vec<BCPackage>> {
    use crate::yaml;
    let packages: BuildConstraints =
        yaml::yaml_from_file(f).unwrap_or_else(|e| panic!("Could not open {f}, error: {e}"));
    packages
        .packages
        .into_iter()
        .filter_map(|(k, v)| {
            if k.contains('@') {
                Some((Maintainer(k), v))
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn test_parse_build_constraints() {
    let _ = parse("test/build-constraints.yaml");
}
