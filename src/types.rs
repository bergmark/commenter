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
pub struct Version(pub String);

impl PartialOrd<Version> for Version {
    fn partial_cmp(&self, r: &Self) -> Option<Ordering> {
        Some(self.cmp(r))
    }
}
impl Ord for Version {
    fn cmp(&self, r: &Self) -> Ordering {
        use Ordering::*;
        let a: Vec<_> = self
            .0
            .split('.')
            .map(|c| c.parse::<usize>().unwrap())
            .collect();
        let b: Vec<_> =
            r.0.split('.')
                .map(|c| c.parse::<usize>().unwrap())
                .collect();
        for (a, b) in a.iter().zip(b.iter()) {
            match a.cmp(b) {
                Less => return Less,
                Greater => return Greater,
                Equal => {}
            }
        }
        a.len().cmp(&b.len())
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for Version {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
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
