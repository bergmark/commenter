use structopt::StructOpt;

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
        Opt::Add => commenter::add::add(),
        Opt::Affected { a, b } => commenter::affected::affected(a, b),
        Opt::Clear => commenter::clear(),
        Opt::DiffSnapshot { a, b } => commenter::snapshot::diff_snapshot(a, b),
        Opt::Disabled => commenter::disabled(),
        Opt::Multiple => commenter::multiple::multiple(),
        Opt::Outdated => commenter::outdated(),
    }
}
