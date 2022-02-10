use crate::build_constraints;
use crate::snapshot::{self, Diff, Snapshot};
use crate::yaml;

pub fn affected(a: String, b: String) {
    let diff = snapshot::to_diff(
        yaml::yaml_from_file(a).unwrap(),
        yaml::yaml_from_file(b).unwrap(),
    );
    affected_impl(diff, "build-constraints.yaml")
}

fn affected_impl(diff: Snapshot, bc: &str) {
    let packages = build_constraints::parse(bc);
    let packages = build_constraints::transpose(packages);
    for (package, diff) in diff.packages {
        match diff {
            Diff::Left(v) => {
                let maintainers = if let Some(maintainers) = packages.get(&package) {
                    maintainers
                        .iter()
                        .map(|m| m.0.clone())
                        .collect::<Vec<_>>()
                        .join(", ")
                } else {
                    "UNMAINTAINED".to_owned()
                };
                println!("{package}-{v}: {maintainers}");
            }
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
        for line in diff.lines() {
            let (c, package, version, new_version) = parse_line(line);
            packages.insert(
                package,
                match &*c {
                    "+" => Diff::Right(version),
                    "-" => Diff::Left(version),
                    "^" | "~" => Diff::Both(version, new_version.unwrap()),
                    _ => unreachable!(),
                },
            );
        }
        Snapshot { packages }
    }

    fn parse_line(line: &str) -> (String, Package, Version, Option<Version>) {
        let r = regex!(
            r#"^(?P<symbol>[+-^~]) (?P<package>.+?)-(?P<version>[.\d]+)(?: -> (?P<new_version>[.\d]+))?$"#
        );
        let caps = r.captures(line).unwrap_or_else(|| panic!("{}", line));
        let c: String = cap_into(&caps, "symbol");
        let package: Package = cap_into(&caps, "package");
        let version: Version = cap_into(&caps, "version");
        let new_version: Option<Version> = cap_into_opt(&caps, "new_version");
        (c, package, version, new_version)
    }

    #[test]
    fn test_parse_line() {
        let s = "~ aeson-1.5.6.0 -> 2.0.3.0";
        parse_line(s);
    }

    #[test]
    fn test_affected() {
        let diff = parse_diff(include_str!("../test/snapshot-diff.txt"));
        affected_impl(diff, "test/build-constraints.yaml")
    }
}
