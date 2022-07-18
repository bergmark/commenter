use crate::prelude::*;

use std::fs::File;
use std::io::{LineWriter, Write};

use crate::regex::*;
use crate::types::*;
use crate::util::fs::read_lines;

pub struct DisabledPackage {
    pub package: Package,
}

pub fn handle<F>(
    build_constraints: &Path,
    write: bool,
    mut f: F,
) -> (Vec<VersionedPackage>, Vec<DisabledPackage>)
where
    F: FnMut(Location, Vec<String>) -> Vec<String>,
{
    let mut new_lines: Vec<String> = vec![];
    let mut versioned_packages: Vec<VersionedPackage> = vec![];
    let mut disabled_packages: Vec<DisabledPackage> = vec![];

    let mut state = State::LookingForLibBounds;
    let mut buf = vec![];
    for line in read_lines(build_constraints).map(|s| s.unwrap()) {
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
        let file = File::create(build_constraints).unwrap();
        let mut file = LineWriter::new(file);

        for line in new_lines {
            file.write_all((line + "\n").as_bytes()).unwrap();
        }
        file.flush().unwrap();
    }

    (versioned_packages, disabled_packages)
}

#[derive(Debug, Copy, Clone)]
enum State {
    LookingForLibBounds,
    ProcessingLibBounds,
    LookingForTestBounds,
    ProcessingTestBounds,
    LookingForBenchBounds,
    ProcessingBenchBounds,
    Done,
}

#[derive(Debug, Copy, Clone)]
pub enum Location {
    Lib,
    Test,
    Bench,
}

fn parse_versioned_package_yaml(s: &str) -> Option<VersionedPackage> {
    if let Ok(cap) = Captures::new(regex!(r#"- *([^ ]+) < *0 *# *([\d.]+)"#), s) {
        let package = cap.get(1).unwrap();
        let version = cap.try_get(2).unwrap();
        Some(VersionedPackage { package, version })
    } else if let Ok(cap) = Captures::new(regex!(r#"- *([^ ]+) *# *([\d.]+)"#), s) {
        let package = cap.get(1).unwrap();
        let version = cap.try_get(2).unwrap();
        Some(VersionedPackage { package, version })
    } else {
        None
    }
}

fn parse_disabled_package(s: &str) -> Option<DisabledPackage> {
    if !regex!(r#"- *([^ ]+) < *0 *# tried"#).is_match(s) {
        Captures::new(regex!(r#"- *([^ ]+) < *0 *# *\d*[^\d ]"#), s)
            .ok()
            .map(|cap| DisabledPackage {
                package: cap.get(1).unwrap(),
            })
    } else {
        None
    }
}
