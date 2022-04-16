use crate::prelude::*;

use crate::snapshot::{to_diff, Diff};
use crate::yaml;

pub fn diff_snapshot(a: &Path, b: &Path) {
    let diff = to_diff(
        yaml::yaml_from_file(a).unwrap(),
        yaml::yaml_from_file(b).unwrap(),
    );
    for (name, diff) in diff.packages {
        let s = match diff {
            Diff::Left(a) => format!("- {name}-{a}"),
            Diff::Right(b) => format!("+ {name}-{b}"),
            Diff::Both(a, b) => format!("^ {name}-{a} -> {b}"),
        };
        println!("{s}");
    }
}
