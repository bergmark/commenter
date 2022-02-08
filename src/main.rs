use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "commenter",
    about = "Automates generation of bounds in  build-constraints.yaml"
)]
enum Opt {
    Add,
    Clear,
    DiffSnapshot { a: String, b: String },
    Disabled,
    Outdated,
}

fn main() {
    let opt = Opt::from_args();
    match opt {
        Opt::Add => commenter::add::add(),
        Opt::Clear => commenter::clear(),
        Opt::DiffSnapshot { a, b } => commenter::snapshot::diff_snapshot(a, b),
        Opt::Disabled => commenter::disabled(),
        Opt::Outdated => commenter::outdated(),
    }
}
