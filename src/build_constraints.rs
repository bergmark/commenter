use crate::prelude::*;

use crate::regex::*;
use crate::types::Package;
use lazy_regex::regex;
use serde::{Deserialize, Deserializer};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Maintenance {
    Maintainer(Maintainer),
    Other(String),
}

impl Maintenance {
    pub fn maintainer(&self) -> Option<&Maintainer> {
        match self {
            Maintenance::Maintainer(m) => Some(m),
            Maintenance::Other(_) => None,
        }
    }
}

impl fmt::Display for Maintenance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Maintenance::Maintainer(m) => m.fmt(f),
            Maintenance::Other(o) => o.fmt(f),
        }
    }
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Debug, Clone)]
pub struct Maintainer(pub String);

impl fmt::Display for Maintainer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
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
}

impl BCPackage {
    fn parse(s: &str) -> Result<BCPackage, anyhow::Error> {
        let r = regex!(r#"^(?P<package>[\da-zA-z][\da-zA-Z-]*) *(?:(?P<bound>.+?))? *$"#);
        let cap = Captures::new(r, s)?;
        let package = cap.name("package")?;
        let bound = cap.name("bound").ok();
        Ok(BCPackage { package, bound })
    }
}

#[test]
fn test_parse_bc_package() {
    fn t(s: &str, package: &str, bound: Option<&str>) {
        assert_eq!(
            BCPackage::parse(s).map_err(|e| e.to_string()),
            Ok(BCPackage {
                package: package.into(),
                bound: bound.map(|b| b.to_owned())
            })
        );
    }

    t("cleff", "cleff", None);
    t("gitlab-haskell < 0", "gitlab-haskell", Some("< 0"));
    t(
        "alex < 3.2.7 || > 3.2.7",
        "alex",
        Some("< 3.2.7 || > 3.2.7"),
    );
}

impl<'de> serde::Deserialize<'de> for BCPackage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = serde::Deserialize::deserialize(deserializer)?;
        BCPackage::parse(&s).map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}

pub struct BuildConstraints {
    pub ghc_version: String,
    pub packages: BTreeMap<Maintenance, Vec<BCPackage>>,
}

pub struct BCPackage2 {
    pub bounds: Vec<String>,
    pub maintainers: Vec<Maintenance>,
}

impl BCPackage2 {
    fn empty() -> BCPackage2 {
        BCPackage2 {
            bounds: vec![],
            maintainers: vec![],
        }
    }
    fn append(&mut self, bound: Option<String>, maintainer: Maintenance) {
        if let Some(bound) = bound {
            self.bounds.push(bound);
        }
        self.maintainers.push(maintainer);
    }
}

pub struct BuildConstraintsByPackage {
    pub ghc_version: String,
    pub packages: BTreeMap<Package, BCPackage2>,
}

impl BuildConstraintsByPackage {
    pub fn package(&self, package: &Package) -> Option<&BCPackage2> {
        self.packages.get(package)
    }
}

impl BuildConstraints {
    pub fn maintainers(&self) -> impl Iterator<Item = &Maintenance> {
        self.packages.keys()
    }

    pub fn by_package(self) -> BuildConstraintsByPackage {
        let BuildConstraints {
            ghc_version,
            packages,
        } = self;
        let mut packages2: BTreeMap<Package, BCPackage2> = BTreeMap::new();
        for (maintainer, packages) in packages {
            for BCPackage { package, bound } in packages {
                packages2
                    .entry(package)
                    .or_insert_with(BCPackage2::empty)
                    .append(bound, maintainer.clone());
            }
        }
        BuildConstraintsByPackage {
            ghc_version,
            packages: packages2,
        }
    }
}

pub fn parse(f: &Path) -> BuildConstraints {
    use crate::yaml;

    #[derive(Deserialize, Debug, PartialEq, Eq)]
    pub struct BuildConstraintsYaml {
        #[serde(rename = "ghc-version")]
        ghc_version: String,
        packages: BTreeMap<String, Vec<BCPackage>>,
    }

    let BuildConstraintsYaml {
        ghc_version,
        packages,
    } = yaml::yaml_from_file(f)
        .unwrap_or_else(|e| panic!("Could not open build-constraints file at {f:?}, error: {e}"));
    let packages = packages
        .into_iter()
        .map(|(k, v)| {
            let maintainer = if [
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
                Maintenance::Other(k)
            } else {
                Maintenance::Maintainer(Maintainer(k))
            };
            (maintainer, v)
        })
        .collect();
    BuildConstraints {
        ghc_version,
        packages,
    }
}

#[test]
fn test_parse_build_constraints() {
    parse(&PathBuf::from("test/build-constraints.yaml"));
}
