use crate::prelude::*;

use crate::snapshot::{to_diff, Diff, Snapshot};
use crate::types::Package;
use crate::util::fs::read_lines;
use crate::yaml;

#[derive(Debug, Clone, Copy, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum Mode {
    Text,
    Cabal,
}

pub fn diff_snapshot(a: &Path, b: &Path, mode: Mode, ignore_file: Option<&Path>) {
    let mut diff = to_diff(
        yaml::yaml_from_file(a).unwrap(),
        yaml::yaml_from_file(b).unwrap(),
    );

    let ignores: HashSet<Package> = if let Some(ignore_file) = ignore_file {
        read_lines(ignore_file).map(|r| r.unwrap().into()).collect()
    } else {
        HashSet::new()
    };

    match mode {
        Mode::Text => {
            diff.packages.retain(|p, _| !ignores.contains(p));

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
            print_cabal_project(diff, ignores);
        }
    }
}

fn print_cabal_project(diff: Snapshot, ignores: HashSet<Package>) {
    println!(
        "cabal-version: 2.4
name: commenter
version: 0
library
  default-language: Haskell2010
  build-depends: base"
    );

    for (name, diff) in diff.packages {
        let s = match diff {
            Diff::Left(_) => None,
            Diff::Right(version) => Some((name, version)),
            Diff::Both(_, version) => Some((name, version)),
        };
        if let Some((name, version)) = s {
            if ignores.contains(&name) {
                println!("      -- , {name} == {version}");
            } else {
                println!("      , {name} == {version}");
            }
        }
    }
}
