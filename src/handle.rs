use std::fs::File;
use std::io::{BufRead, BufReader, LineWriter, Lines, Write};
use std::path::Path;

use crate::prelude::*;
use crate::types::*;

pub struct DisabledPackage {
    pub package: String,
}

pub fn handle<F>(write: bool, mut f: F) -> (Vec<VersionedPackage>, Vec<DisabledPackage>)
where
    F: FnMut(Location, Vec<String>) -> Vec<String>,
{
    let path = "build-constraints.yaml";
    let mut new_lines: Vec<String> = vec![];
    let mut versioned_packages: Vec<VersionedPackage> = vec![];
    let mut disabled_packages: Vec<DisabledPackage> = vec![];

    let mut state = State::LookingForLibBounds;
    let mut buf = vec![];
    for line in read_lines(path).map(|s| s.unwrap()) {
        if let Some(versioned_package) = parse_versioned_package_yaml(&line) {
            versioned_packages.push(versioned_package);
        } else if let Some(disabled_package) = parse_disabled_package(&line) {
            disabled_packages.push(disabled_package);
        }

        match state {
            State::LookingForLibBounds => {
                if line == r#"    "Library and exe bounds failures":"# {
                    state = State::ProcessingLibBounds;
                }
                new_lines.push(line);
            }
            State::ProcessingLibBounds => {
                if line == r#"    # End of Library and exe bounds failures"# {
                    new_lines.extend(f(Location::Lib, buf).into_iter());
                    buf = vec![];
                    new_lines.push(line);
                    state = State::LookingForTestBounds;
                } else {
                    // Remove empty section
                    if line != "        []" {
                        buf.push(line);
                    }
                }
            }
            State::LookingForTestBounds => {
                if line == r#"    # Test bounds issues"# {
                    state = State::ProcessingTestBounds;
                }
                new_lines.push(line);
            }
            State::ProcessingTestBounds => {
                if line == r#"    # End of Test bounds issues"# {
                    new_lines.extend(f(Location::Test, buf).into_iter());
                    buf = vec![];
                    new_lines.push(line);
                    state = State::LookingForBenchBounds;
                } else {
                    buf.push(line);
                }
            }
            State::LookingForBenchBounds => {
                if line == r#"    # Benchmark bounds issues"# {
                    state = State::ProcessingBenchBounds;
                }
                new_lines.push(line);
            }
            State::ProcessingBenchBounds => {
                if line == r#"    # End of Benchmark bounds issues"# {
                    new_lines.extend(f(Location::Bench, buf).into_iter());
                    buf = vec![];
                    new_lines.push(line);
                    state = State::Done;
                } else {
                    buf.push(line);
                }
            }
            State::Done => {
                new_lines.push(line);
            }
        }
    }

    if write {
        let file = File::create(path).unwrap();
        let mut file = LineWriter::new(file);

        for line in new_lines {
            file.write_all((line + "\n").as_bytes()).unwrap();
        }
        file.flush().unwrap();
    }

    (versioned_packages, disabled_packages)
}

enum State {
    LookingForLibBounds,
    ProcessingLibBounds,
    LookingForTestBounds,
    ProcessingTestBounds,
    LookingForBenchBounds,
    ProcessingBenchBounds,
    Done,
}

pub enum Location {
    Lib,
    Test,
    Bench,
}

fn read_lines<P>(filename: P) -> Lines<BufReader<File>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename).unwrap();
    BufReader::new(file).lines()
}

fn parse_versioned_package_yaml(s: &str) -> Option<VersionedPackage> {
    if let Some(caps) = regex!(r#"- *([^ ]+) < *0 *# *([\d.]+)"#).captures(s) {
        let package = Package(caps.get(1).unwrap().as_str().to_owned());
        let version = Version(caps.get(2).unwrap().as_str().to_owned());
        Some(VersionedPackage { package, version })
    } else if let Some(caps) = regex!(r#"- *([^ ]+) *# *([\d.]+)"#).captures(s) {
        let package = Package(caps.get(1).unwrap().as_str().to_owned());
        let version = Version(caps.get(2).unwrap().as_str().to_owned());
        Some(VersionedPackage { package, version })
    } else {
        None
    }
}

fn parse_disabled_package(s: &str) -> Option<DisabledPackage> {
    if !regex!(r#"- *([^ ]+) < *0 *# tried"#).is_match(s) {
        if let Some(caps) = regex!(r#"- *([^ ]+) < *0 *# *\d*[^\d ]"#).captures(s) {
            let package = caps.get(1).unwrap().as_str().to_owned();
            Some(DisabledPackage { package })
        } else {
            None
        }
    } else {
        None
    }
}