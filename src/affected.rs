use crate::snapshot::{self, Diff, Snapshot};
use crate::yaml;

pub fn affected(a: String, b: String) {
    let diff = snapshot::to_diff(
        yaml::yaml_from_file(a).unwrap(),
        yaml::yaml_from_file(b).unwrap(),
    );
    affected_impl(diff)
}

fn affected_impl(diff: Snapshot) {
    for (_name, diff) in diff.packages {
        let _s = match diff {
            Diff::Left(_) => {}
            Diff::Right(_) | Diff::Both(_, _) => {}
        };
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::prelude::*;
    use crate::regex::{cap_into, cap_into_opt};
    use crate::snapshot::{Diff, Snapshot};
    use crate::types::{Package, Version};

    fn parse_diff(diff: &str) -> Snapshot {
        let mut packages: BTreeMap<_, _> = Default::default();
        let r = regex!(
            r#"^(?P<symbol>[+-^]) (?P<package>.+?)-(?P<version>[.\d]+)(:? -> (?P<new_version>[\.d]+))?$"#
        );
        for line in diff.lines() {
            let caps = r.captures(line).unwrap();
            let c: String = cap_into(&caps, "symbol");
            let package: Package = cap_into(&caps, "package");
            let version: Version = cap_into(&caps, "version");
            let new_version: Option<Version> = cap_into_opt(&caps, "new_version");
            packages.insert(
                package,
                match &*c {
                    "+" => Diff::Right(version),
                    "-" => Diff::Left(version),
                    "^" => Diff::Both(version, new_version.unwrap()),
                    _ => unreachable!(),
                },
            );
        }
        Snapshot { packages }
    }

    #[test]
    #[ignore]
    fn test_affected() {
        let diff = parse_diff(include_str!("../test/snapshot-diff.txt"));
        affected_impl(diff)
    }
}
