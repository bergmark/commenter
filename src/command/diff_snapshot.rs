use crate::prelude::*;

use crate::snapshot::{to_diff, Diff, Snapshot};
use crate::yaml;

#[derive(Debug, Clone, Copy, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum Mode {
    Text,
    Cabal,
}

pub fn diff_snapshot(a: &Path, b: &Path, mode: Mode) {
    let diff = to_diff(
        yaml::yaml_from_file(a).unwrap(),
        yaml::yaml_from_file(b).unwrap(),
    );
    match mode {
        Mode::Text => {
            for (name, diff) in diff.packages {
                let s = match diff {
                    Diff::Left(a) => format!("- {name}-{a}"),
                    Diff::Right(b) => format!("+ {name}-{b}"),
                    Diff::Both(a, b) => format!("^ {name}-{a} -> {b}"),
                };
                println!("{s}");
            }
        }
        Mode::Cabal => {
            print_cabal_project(diff);
        }
    }
}

fn print_cabal_project(diff: Snapshot) {
    println!(
        "cabal-version:      2.4
name:               commenter
version:            0
executable commenter-test-diff
    build-depends:    base"
    );

    for (name, diff) in diff.packages {
        let s = match diff {
            Diff::Left(_) => None,
            Diff::Right(b) => Some(format!("      , {name} == {b}")),
            Diff::Both(_, b) => Some(format!("      , {name} == {b}")),
        };
        if let Some(s) = s {
            println!("{s}");
        }
    }
}
