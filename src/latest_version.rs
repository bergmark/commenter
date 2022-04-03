use crate::prelude::*;

use rusqlite::Connection;

use crate::types::*;

pub fn latest_version(packages: impl Iterator<Item = Package>) -> BTreeMap<Package, Version> {
    let all_versions = all_versions(packages);
    all_versions
        .into_iter()
        .map(|(p, vs)| (p, vs.into_iter().max().unwrap()))
        .collect()
}

fn all_versions(packages: impl Iterator<Item = Package>) -> BTreeMap<Package, BTreeSet<Version>> {
    let tilde = home::home_dir().unwrap_or_else(|| panic!("Could not find $HOME"));
    let mut res = BTreeMap::new();
    let conn = Connection::open(format!(
        "{tilde}/.stack/pantry/pantry.sqlite3",
        tilde = tilde.display()
    ))
    .unwrap();
    for package in packages {
        let mut stmt = conn.prepare(
            &format!("select version.version, revision from hackage_cabal as h, version where h.name=(select id from package_name where name = '{package}') and h.version=version.id")
        ).unwrap();

        let version_revisions = stmt
            .query_map([], |row| {
                Ok(VersionRevision {
                    version: row.get(0).unwrap(),
                    revision: row.get(1).unwrap(),
                })
            })
            .unwrap();

        for v in version_revisions {
            let VersionRevision {
                version,
                revision: _,
            } = v.unwrap();
            let e = res.entry(package.clone()).or_insert_with(BTreeSet::new);
            e.insert(Version(version));
        }
    }
    res
}

#[derive(Debug)]
pub struct VersionRevision {
    pub version: String,
    pub revision: usize,
}
