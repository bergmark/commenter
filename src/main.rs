use std::path::PathBuf;

use structopt::StructOpt;

use commenter::command;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "commenter",
    about = "Automates generation of bounds in  build-constraints.yaml"
)]
enum Opt {
    Add {
        #[structopt(short, long, default_value = "build-constraints.yaml")]
        build_constraints: PathBuf,
    },
    AddLoop {
        #[structopt(short, long, default_value = "build-constraints.yaml")]
        build_constraints: PathBuf,
        #[structopt(long)]
        clear: bool,
    },
    Affected {
        #[structopt(short, long, default_value = "build-constraints.yaml")]
        build_constraints: PathBuf,
        older: String,
        newer: String,
    },
    Clear {
        #[structopt(short, long, default_value = "build-constraints.yaml")]
        build_constraints: PathBuf,
    },
    DiffSnapshot {
        older: String,
        newer: String,
    },
    Disabled {
        #[structopt(short, long, default_value = "build-constraints.yaml")]
        build_constraints: PathBuf,
    },
    Maintainers {
        #[structopt(short, long, default_value = "build-constraints.yaml")]
        build_constraints: PathBuf,
    },
    Multiple {
        #[structopt(short, long, default_value = "build-constraints.yaml")]
        build_constraints: PathBuf,
    },
    Outdated {
        #[structopt(short, long, default_value = "build-constraints.yaml")]
        build_constraints: PathBuf,
    },
}

fn main() {
    let opt = Opt::from_args();
    match opt {
        Opt::Add { build_constraints } => command::add::add(&build_constraints),
        Opt::AddLoop {
            build_constraints,
            clear,
        } => command::add_loop::add_loop(&build_constraints, clear),
        Opt::Affected {
            build_constraints,
            older,
            newer,
        } => command::affected::affected(&build_constraints, older, newer),
        Opt::Clear { build_constraints } => command::clear(&build_constraints),
        Opt::DiffSnapshot { older, newer } => command::diff_snapshot::diff_snapshot(older, newer),
        Opt::Disabled { build_constraints } => command::disabled::disabled(&build_constraints),
        Opt::Maintainers { build_constraints } => {
            command::maintainers::maintainers(&build_constraints)
        }
        Opt::Multiple { build_constraints } => command::multiple::multiple(&build_constraints),
        Opt::Outdated { build_constraints } => command::outdated::outdated(&build_constraints),
    }
}
