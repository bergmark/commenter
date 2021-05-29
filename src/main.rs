use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

use regex::Regex;

type H = HashMap<String, Vec<(String, String)>>;

fn main() {
    let mut lib_exes: H = Default::default();
    let mut tests: H = Default::default();
    let mut benches: H = Default::default();
    let mut last_header: Option<String> = None;
    let empty = Regex::new(r#"^\s+$"#).unwrap();
    let header = Regex::new(r#"^(?P<header>\w.+)$"#).unwrap();
    let package =
        Regex::new(r#"^- \[ \] (?P<package>[a-zA-z]([a-zA-z0-9.-]+?))(?P<rest>-(\d+(\.\d+)*) .+\. Used by: (?P<component>.+))$"#)
            .unwrap();

    if let Ok(lines) = read_lines("./comments.txt") {
        for line in lines {
            if let Ok(line) = line {
                if empty.captures(&line).is_some() {
                } else if let Some(cap) = package.captures(&line) {
                    let root = last_header.clone().unwrap();
                    let component = cap.name("component").unwrap().as_str();
                    let package = cap.name("package").unwrap().as_str();
                    let rest = cap.name("rest").unwrap().as_str();
                    match component {
                        "library" | "executable" => insert(&mut lib_exes, root, package, rest),
                        "benchmark" => insert(&mut benches, root, package, rest),
                        "test-suite" => insert(&mut tests, root, package, rest),
                        _ => panic!("Bad component: {}", component),
                    }
                } else if let Some(cap) = header.captures(&line) {
                    let header = cap.name("header").unwrap().as_str();
                    last_header = Some(header.to_owned());
                }
            }
        }
    }

    if !lib_exes.is_empty() {
        println!("\nLIBS + EXES\n");
    }
    for (header, packages) in lib_exes {
        println!("        # {header}", header = header,);
        for (package, rest) in packages {
            println!(
                "        - {package} < 0 # {rest}",
                package = package,
                rest = rest
            );
        }
    }

    if !tests.is_empty() {
        println!("\nTESTS\n");
    }
    for (header, packages) in tests {
        println!("    # {header}", header = header,);
        for (package, rest) in packages {
            println!("    - {package} # {rest}", package = package, rest = rest);
        }
    }

    if !benches.is_empty() {
        println!("\nBENCHMARKS\n");
    }
    for (header, packages) in benches {
        println!("    # {header}", header = header,);
        for (package, rest) in packages {
            println!("    - {package} # {rest}", package = package, rest = rest);
        }
    }
}

fn insert(h: &mut H, root: String, package: &str, rest: &str) {
    (*h.entry(root).or_insert_with(|| vec![])).push((package.to_owned(), rest.to_owned()));
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
