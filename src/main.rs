use std::path::PathBuf;

use clap::Parser;

use commenter::command;

#[derive(Debug, Parser)]
#[structopt(
    name = "commenter",
    about = "Automates operations on Stackage's build-constraints.yaml"
)]
enum Opt {
    /// Reads `curator` bounds failures from from stdin and disabled packages accordingly.
    Add {
        #[structopt(short, long, default_value = "build-constraints.yaml")]
        build_constraints: PathBuf,
    },
    /// Like `add` but runs curator internally, looping until there
    /// are no more bounds failures.
    ///
    /// Pass `--clear` to remove all
    /// generated bounds (updating anything that is out of date).
    AddLoop {
        #[structopt(short, long, default_value = "build-constraints.yaml")]
        build_constraints: PathBuf,
        #[structopt(long)]
        clear: bool,
        #[structopt(long)]
        target: Option<String>,
    },
    /// Takes the diff of two snapshots and produces packages +
    /// maintainers of any removed packages, to be able to ping all
    /// affected maintainers.
    Affected {
        #[structopt(short, long, default_value = "build-constraints.yaml")]
        build_constraints: PathBuf,
        older: PathBuf,
        newer: PathBuf,
    },
    /// Removes all bounds that were generated by `add` from build-constraints
    Clear {
        #[structopt(short, long, default_value = "build-constraints.yaml")]
        build_constraints: PathBuf,
    },
    /// Produces a diff between two snapshots, showing added, removed,
    /// and up/down-graded packages.
    DiffSnapshot {
        older: PathBuf,
        newer: PathBuf,
        #[structopt(long, default_value = "text")]
        mode: crate::command::diff_snapshot::Mode,
        #[structopt(long)]
        ignore_file: Option<PathBuf>,
    },
    /// Print the number of reverse dependencies that are blocked by disabled packages
    Disabled {
        #[structopt(short, long, default_value = "build-constraints.yaml")]
        build_constraints: PathBuf,
    },
    /// Prints packages that are mentioned in comments but not
    /// elsewhere in a format that can be pasted into Grandfathered
    /// Dependencies.
    Grandfather {
        #[structopt(short, long, default_value = "build-constraints.yaml")]
        build_constraints: PathBuf,
    },
    /// Prints maintainer sections with missing github handles
    Maintainers {
        #[structopt(short, long, default_value = "build-constraints.yaml")]
        build_constraints: PathBuf,
    },
    /// Prints packages that are part of multiple maintainer sections
    Multiple {
        #[structopt(short, long, default_value = "build-constraints.yaml")]
        build_constraints: PathBuf,
    },
    /// Finds mentioned package versions that are out of date.
    Outdated {
        #[structopt(short, long, default_value = "build-constraints.yaml")]
        build_constraints: PathBuf,
        #[structopt(long)]
        ignore_file: Option<PathBuf>,
        /// Show lines matching the package name in build-constraints
        #[structopt(long)]
        show_lines: bool,
    },
    /// Dig out info about a package. The snapshot queries take a lot of time.
    PackageInfo {
        #[structopt(short, long, default_value = "../stackage-snapshots")]
        stackage_snapshots_path: PathBuf,
        #[structopt(short, long, default_value = "build-constraints.yaml")]
        build_constraints: PathBuf,
        #[structopt(short, long)]
        no_search_snapshots: bool,
        package: String,
    },
}

fn main() {
    let opt = Opt::parse();
    match opt {
        Opt::Add { build_constraints } => command::add::add(&build_constraints),
        Opt::AddLoop {
            build_constraints,
            clear,
            target,
        } => command::add_loop::add_loop(&build_constraints, clear, target),
        Opt::Affected {
            build_constraints,
            older,
            newer,
        } => command::affected::affected(&build_constraints, &older, &newer),
        Opt::Clear { build_constraints } => command::clear(&build_constraints),
        Opt::DiffSnapshot {
            older,
            newer,
            mode,
            ignore_file,
        } => command::diff_snapshot::diff_snapshot(&older, &newer, mode, ignore_file.as_deref()),
        Opt::Disabled { build_constraints } => command::disabled::disabled(&build_constraints),
        Opt::Grandfather { build_constraints } => {
            command::grandfather::grandfather(&build_constraints)
        }
        Opt::Maintainers { build_constraints } => {
            command::maintainers::maintainers(&build_constraints)
        }
        Opt::Multiple { build_constraints } => command::multiple::multiple(&build_constraints),
        Opt::Outdated {
            build_constraints,
            ignore_file,
            show_lines,
        } => command::outdated::outdated(&build_constraints, ignore_file.as_deref(), show_lines),
        Opt::PackageInfo {
            stackage_snapshots_path,
            no_search_snapshots,
            build_constraints,
            package,
        } => command::package_info::package_info(
            &stackage_snapshots_path,
            no_search_snapshots,
            &build_constraints,
            &package,
        ),
    }
}
