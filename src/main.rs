use structopt::StructOpt;

use commenter::command;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "commenter",
    about = "Automates generation of bounds in  build-constraints.yaml"
)]
enum Opt {
    Add,
    Affected { a: String, b: String },
    Clear,
    DiffSnapshot { a: String, b: String },
    Disabled,
    Multiple,
    Outdated,
}

fn main() {
    let opt = Opt::from_args();
    match opt {
        Opt::Add => command::add::add(),
        Opt::Affected { a, b } => command::affected::affected(a, b),
        Opt::Clear => command::clear(),
        Opt::DiffSnapshot { a, b } => command::diff_snapshot::diff_snapshot(a, b),
        Opt::Disabled => command::disabled::disabled(),
        Opt::Multiple => command::multiple::multiple(),
        Opt::Outdated => command::outdated::outdated(),
    }
}
