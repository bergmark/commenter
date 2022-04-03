use crate::prelude::*;

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Debug, Hash)]
pub struct Package(pub String);

impl fmt::Display for Package {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for Package {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Version(Vec<usize>);

impl PartialOrd<Version> for Version {
    fn partial_cmp(&self, r: &Self) -> Option<Ordering> {
        Some(self.cmp(r))
    }
}
impl Ord for Version {
    fn cmp(&self, r: &Self) -> Ordering {
        use Ordering::*;
        for (a, b) in self.0.iter().zip(r.0.iter()) {
            match a.cmp(b) {
                Less => return Less,
                Greater => return Greater,
                Equal => {}
            }
        }
        self.0.len().cmp(&r.0.len())
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s: String = self
            .0
            .iter()
            .map(|c| format!("{c}"))
            .collect::<Vec<String>>()
            .join(".");
        write!(f, "{s}")
    }
}

impl TryFrom<&str> for Version {
    type Error = std::num::ParseIntError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.split('.')
            .map(|s| s.parse::<usize>())
            .collect::<Result<_, _>>()
            .map(Version)
    }
}

#[derive(Clone, PartialOrd, Ord, PartialEq, Eq, Debug, Hash)]
pub struct Revision(pub usize);

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub struct VersionedPackage {
    pub package: Package,
    pub version: Version,
}

impl fmt::Display for VersionedPackage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { package, version } = self;
        write!(f, "{package}-{version}")
    }
}
