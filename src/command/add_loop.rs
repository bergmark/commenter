use crate::prelude::*;

use crate::build_constraints;
use crate::command;
use crate::curator;

pub fn add_loop(build_constraints: &Path, clear: bool, target: Option<String>) {
    if clear {
        println!("Clearing {}", build_constraints.display());
        crate::command::clear(build_constraints);
    }

    let bc = build_constraints::parse(build_constraints);
    let ghc_version = bc.ghc_version;

    let target = target.unwrap_or_else(|| {
        let datetime = chrono::Utc::now().format("%Y-%m-%d");
        format!("nightly-{datetime}")
    });

    let no_download = target.starts_with("lts-");

    {
        println!("curator update");
        curator::update();
    }

    let mut add = true;

    while add {
        println!("curator constraints");
        curator::constraints(&target, no_download);
        println!("curator snapshot-incomplete");
        curator::snapshot_incomplete(&target);
        println!("curator snapshot");
        curator::snapshot();

        let lines = curator::stack(&ghc_version);
        let lib_count = command::add::add_impl(build_constraints, move || lines);
        if lib_count == 0 {
            add = false;
        }
    }

    println!("Done!");
}
