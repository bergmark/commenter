use crate::prelude::*;

use crate::handle::{handle, Location};
use crate::regex::*;
use crate::types::*;

pub(crate) type M = BTreeMap<Package, (Vec<VersionedPackage>, Option<usize>)>;

pub fn disabled(build_constraints: &Path) {
    let mut disabled_transitively: Vec<DisabledTransitively> = vec![];
    handle(build_constraints, false, |loc, lines| {
        match loc {
            Location::Lib => disabled_transitively.extend(
                lines
                    .into_iter()
                    .filter_map(|line| parse_disabled_transitviely(&line))
                    .collect::<Vec<_>>(),
            ),
            Location::Test | Location::Bench => (),
        }
        vec![]
    });

    let mut packages: BTreeSet<Package> = BTreeSet::new();
    let mut disabled: M = BTreeMap::new();

    for DisabledTransitively { child, parent } in disabled_transitively {
        packages.insert(child.package.clone());
        packages.insert(parent.clone());
        disabled
            .entry(child.package.clone())
            .or_insert_with(|| (vec![], None));
        let t = disabled.entry(parent).or_insert_with(|| (vec![], None));
        t.0.push(child);
    }

    let mut packages_len = packages.len();
    while packages_len > 0 {
        let mut new_packages: BTreeSet<Package> = BTreeSet::new();
        for package in packages {
            let (_, count) = disabled.get(&package).unwrap();
            if count.is_none() && !process(&package, &mut disabled) {
                new_packages.insert(package.clone());
            }
        }
        packages = new_packages;
        packages_len = packages.len();
    }

    let mut v: Vec<_> = disabled
        .into_iter()
        .map(|(package, (_, count))| (count, package))
        .collect();
    v.sort();
    for (count, package) in v {
        let count = count.unwrap();
        if count != 0 {
            println!("{package} is disabled with {count} dependents");
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub(crate) struct DisabledTransitively {
    pub(crate) child: VersionedPackage,
    pub(crate) parent: Package,
}

pub(crate) fn parse_disabled_transitviely(s: &str) -> Option<DisabledTransitively> {
    let r = regex!(
        r#"- *([^ ]+) < *0 *# tried [^ ]+-([\d.]+), but its \*[^*]+\* requires the disabled package: ([^ ]+)"#
    );
    Captures::new(r, s).ok().map(|cap| {
        let package = cap.get(1).unwrap();
        let version = cap.try_get(2).unwrap();
        let parent = cap.try_get(3).unwrap();
        DisabledTransitively {
            child: VersionedPackage { package, version },
            parent,
        }
    })
}

#[test]
fn test_parse_disabled_transitviely() {
    let s = "- Network-NineP < 0 # tried Network-NineP-0.4.7.1, but its *library* requires the disabled package: mstate";
    assert_eq!(
        parse_disabled_transitviely(s),
        Some(DisabledTransitively {
            child: VersionedPackage {
                package: Package("Network-NineP".to_owned()),
                version: "0.4.7.1".try_into().unwrap()
            },
            parent: Package("mstate".to_owned()),
        })
    )
}

fn process(package: &Package, m: &mut M) -> bool {
    let (children, count) = m.get(package).unwrap();
    assert!(count.is_none(), "{:?}", package);
    let mut count = 0;
    for child in children {
        let (_, child_count) = m
            .get(&child.package)
            .unwrap_or_else(|| panic!("{}", child.package));
        match child_count {
            None => return false,
            Some(i) => count += 1 + i,
        }
    }
    m.entry(package.clone())
        .and_modify(|tup| tup.1 = Some(count))
        .or_insert_with(|| panic!("{}", package));
    true
}
