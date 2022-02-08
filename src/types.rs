use std::fmt;

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Debug, Hash)]
pub struct Package(pub String);

impl fmt::Display for Package {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<&str> for Package {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

#[derive(Clone, PartialOrd, Ord, PartialEq, Eq, Debug, Hash)]
pub struct Version(pub String);

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<&str> for Version {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

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
