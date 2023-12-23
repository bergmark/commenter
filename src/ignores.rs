use crate::prelude::*;

use crate::types::*;
use crate::util::fs::read_lines;

#[derive(Default, Debug)]
pub(crate) struct Ignores {
    versioned: HashSet<VersionedPackage>,
    unversioned: HashSet<Package>,
}

impl Ignores {
    pub(crate) fn from_path(ignore_file: Option<&Path>) -> Self {
        let mut i = Self::default();
        if let Some(ignore_file) = ignore_file {
            for line in read_lines(ignore_file) {
                let line = line.unwrap();
                if let Ok(v) = VersionedPackage::try_from(line.clone()) {
                    i.versioned.insert(v);
                } else {
                    i.unversioned.insert(Package::from(line));
                }
            }
        }
        i
    }

    pub(crate) fn contains<'a, 'b>(
        &'a self,
        v: impl Into<Either<'b, VersionedPackage, Package>>,
    ) -> bool {
        match v.into() {
            Either::Left(v) => self.contains_versioned(v),
            Either::Right(v) => self.contains_unversioned(v),
        }
    }

    fn contains_versioned(&self, v: &VersionedPackage) -> bool {
        self.versioned.contains(v) || self.contains_unversioned(&v.package)
    }

    fn contains_unversioned(&self, v: &Package) -> bool {
        self.unversioned.contains(v)
    }
}

pub(crate) enum Either<'a, L, R> {
    Left(&'a L),
    Right(&'a R),
}

impl<'a> From<&'a VersionedPackage> for Either<'a, VersionedPackage, Package> {
    fn from(v: &'a VersionedPackage) -> Self {
        Either::Left(v)
    }
}

impl<'a> From<&'a Package> for Either<'a, VersionedPackage, Package> {
    fn from(v: &'a Package) -> Self {
        Either::Right(v)
    }
}
