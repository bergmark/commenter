use crate::prelude::Path;

use crate::build_constraints;
use crate::snapshot::{self, Diff, Snapshot};
use crate::yaml;

pub fn affected(build_constraints: &Path, a: &Path, b: &Path) {
    let diff = snapshot::to_diff(
        yaml::yaml_from_file(a).unwrap(),
        yaml::yaml_from_file(b).unwrap(),
    );
    affected_impl(diff, build_constraints)
}

fn affected_impl(diff: Snapshot, bc: &Path) {
    let packages = build_constraints::parse(bc).by_package().packages;
    for (package, diff) in diff.packages {
        match diff {
            Diff::Left(v) => {
                let maintainers = if let Some(package) = packages.get(&package) {
                    package
                        .maintainers
                        .iter()
                        .map(|m| m.to_string())
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
    use crate::regex::*;
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
        let cap = Captures::new(r, line).unwrap();
        let c: String = cap.name("symbol").unwrap();
        let package: Package = cap.name("package").unwrap();
        let version: Version = cap.try_name("version").unwrap();
        let new_version: Option<Version> = cap.try_name("new_version").ok();
        (c, package, version, new_version)
    }

    #[test]
    fn test_parse_line() {
        let s = "~ aeson-1.5.6.0 -> 2.0.3.0";
        parse_line(s);
    }

    #[test]
    fn test_affected() {
        let diff = parse_diff(include_str!("../../test/snapshot-diff.txt"));
        affected_impl(diff, &PathBuf::from("test/build-constraints.yaml"))
    }
}
