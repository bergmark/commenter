use std::io::{self, BufRead};

use crate::handle::{handle, Location};
use crate::prelude::*;
use crate::regex::cap_into;
use crate::types::{Package, Version, VersionedPackage};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Header {
    Versioned(VersionedPackage),
    Missing(Package),
}

#[test]
fn test_parse_package_with_component() {
    let line = "- [ ] captcha-2captcha-0.1.0.0 (==0.1.*). Edward Yang <qwbarch@gmail.com> @qwbarch. @qwbarch. Used by: library";
    assert_eq!(
        parse_package_with_component(line).unwrap(),
        PackageWithComponent {
            package: "captcha-2captcha".into(),
            version: "0.1.0.0".into(),
            component: "library".into(),
        }
    );

    let line =
        "- [ ] b9-3.2.0 (==1.4.*). Sven Heyll <svh@posteo.de> @sheyll. @sheyll. Used by: library";
    assert_eq!(
        parse_package_with_component(line).unwrap(),
        PackageWithComponent {
            package: "b9".into(),
            version: "3.2.0".into(),
            component: "library".into(),
        }
    );
}

#[derive(Debug, PartialEq, Eq)]
struct PackageWithComponent {
    package: Package,
    version: Version,
    component: String,
}

fn parse_package_with_component(s: &str) -> Option<PackageWithComponent> {
    let package = regex!(
        r#"^- \[ \] (?P<package>[\da-zA-z][\da-zA-Z-]*?)-(?P<version>(\d+(\.\d+)*)) \(.+?Used by: (?P<component>.+)$"#
    );
    package.captures(s).map(|cap| PackageWithComponent {
        package: cap_into(&cap, "package"),
        version: cap_into(&cap, "version"),
        component: cap_into(&cap, "component"),
    })
}

#[test]
fn test_parse_header_versioned() {
    let s = "aeson-2.0.3.0 ([changelog](http://hackage.haskell.org/package/aeson-2.0.3.0/changelog)) (Adam Bergmark <adam@bergmark.nl> @bergmark, Stackage upper bounds) is out of bounds for:";
    assert_eq!(
        parse_header_versioned(s).unwrap(),
        Header::Versioned(VersionedPackage {
            package: "aeson".into(),
            version: "2.0.3.0".into(),
        })
    )
}

fn parse_header_versioned(s: &str) -> Option<Header> {
    let header_versioned = regex!(
        r#"^(?P<package>[\da-zA-z][\da-zA-Z-]*?)-(?P<version>(\d+(\.\d+)*)).+?is out of bounds for:$"#
    );
    header_versioned.captures(s).map(|cap| {
        Header::Versioned(VersionedPackage {
            package: cap_into(&cap, "package"),
            version: cap_into(&cap, "version"),
        })
    })
}

#[test]
fn test_parse_header_missing() {
    let s = "n2o-protocols (Compilation failures, Marat Khafizov <xafizoff@gmail.com> @xafizoff) (not present) depended on by:";
    assert_eq!(
        parse_header_missing(s).unwrap(),
        Header::Missing(Package("n2o-protocols".to_owned()))
    );
}

fn parse_header_missing(s: &str) -> Option<Header> {
    let header_missing = regex!(r#"^(?P<package>[\da-zA-z][\da-zA-Z-]*?) .+?depended on by:$"#);

    header_missing
        .captures(s)
        .map(|cap| Header::Missing(cap_into(&cap, "package")))
}

type H = HashMap<Header, Vec<(Package, Version, String)>>;

pub fn add() {
    let mut lib_exes: H = Default::default();
    let mut tests: H = Default::default();
    let mut benches: H = Default::default();
    let mut last_header: Option<Header> = None;

    // Ignore everything until the bounds issues show up.
    let mut process_line = false;

    for line in io::stdin().lock().lines().flatten() {
        if regex!(r#"^\s*$"#).is_match(&line) {
            // noop
        } else if line == "curator: Snapshot dependency graph contains errors:" {
            process_line = true;
        } else if !process_line {
            println!("[INFO] {line}");
        } else if let Some(PackageWithComponent {
            package,
            version,
            component,
        }) = parse_package_with_component(&line)
        {
            let root = last_header.clone().unwrap();
            match &*component {
                "library" | "executable" => {
                    insert(&mut lib_exes, root, &package, &version, &component)
                }
                "benchmark" => insert(&mut benches, root, &package, &version, "benchmarks"),
                "test-suite" => insert(&mut tests, root, &package, &version, &component),
                _ => panic!("Bad component: {component}"),
            }
        } else if let Some(header_versioned) = parse_header_versioned(&line) {
            last_header = Some(header_versioned);
        } else if let Some(missing) = parse_header_missing(&line) {
            last_header = Some(missing);
        } else {
            panic!("Unhandled: {}", line);
        }
    }

    let mut auto_lib_exes = vec![];
    let mut auto_tests = vec![];
    let mut auto_benches = vec![];

    if !lib_exes.is_empty() {
        println!("\nLIBS + EXES\n");
    }
    for (header, packages) in lib_exes {
        for (package, version, component) in packages {
            let s = printer("        ", &package, true, &version, &component, &header);
            println!("{s}");
            auto_lib_exes.push(s);
        }
    }

    if !tests.is_empty() {
        println!("\nTESTS\n");
    }
    for (header, packages) in tests {
        for (package, version, component) in packages {
            let s = printer("    ", &package, false, &version, &component, &header);
            println!("{s}");
            auto_tests.push(s);
        }
    }

    if !benches.is_empty() {
        println!("\nBENCHMARKS\n");
    }
    for (header, packages) in benches {
        for (package, version, component) in packages {
            let s = printer("    ", &package, false, &version, &component, &header);
            println!("{s}");
            auto_benches.push(s);
        }
    }

    println!();
    println!(
        "Adding {lib_exes} libs, {tests} tests, {benches} benches to build-constraints.yaml",
        lib_exes = auto_lib_exes.len(),
        tests = auto_tests.len(),
        benches = auto_benches.len()
    );
    adder(auto_lib_exes, auto_tests, auto_benches);
}

fn printer(
    indent: &str,
    package: &Package,
    lt0: bool,
    version: &Version,
    component: &str,
    header: &Header,
) -> String {
    let lt0 = if lt0 { " < 0" } else { "" };
    format!(
        "{indent}- {package}{lt0} # tried {package}-{version}, but its *{component}* {cause}",
        cause = match header {
            Header::Versioned(versioned) => format!("does not support: {versioned}"),
            Header::Missing(package) => format!("requires the disabled package: {package}"),
        },
    )
}

fn insert(h: &mut H, header: Header, package: &Package, version: &Version, component: &str) {
    (*h.entry(header).or_insert_with(Vec::new)).push((
        package.clone(),
        version.clone(),
        component.to_owned(),
    ));
}

fn adder(lib: Vec<String>, test: Vec<String>, bench: Vec<String>) {
    handle(true, |loc, mut lines| {
        lines.extend(match loc {
            Location::Lib => lib.clone(),
            Location::Test => test.clone(),
            Location::Bench => bench.clone(),
        });
        lines.sort();
        lines
    });
}
