use std::io::{self, BufRead};

use crate::handle::{handle, Location};
use crate::prelude::*;
use crate::regex::*;
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
            version: "0.1.0.0".try_into().unwrap(),
            bound: "==0.1.*".into(),
            component: "library".into(),
        }
    );

    let line =
        "- [ ] b9-3.2.0 (==1.4.*). Sven Heyll <svh@posteo.de> @sheyll. @sheyll. Used by: library";
    assert_eq!(
        parse_package_with_component(line).unwrap(),
        PackageWithComponent {
            package: "b9".into(),
            version: "3.2.0".try_into().unwrap(),
            bound: "==1.4.*".into(),
            component: "library".into(),
        }
    );

    let line = "- [ ] BlastHTTP-1.4.2 (==0.3.3.*). Ketil Malde @ketil-malde. Used by: library";
    assert_eq!(
        parse_package_with_component(line).unwrap(),
        PackageWithComponent {
            package: "BlastHTTP".into(),
            version: "1.4.2".try_into().unwrap(),
            bound: "==0.3.3.*".into(),
            component: "library".into(),
        }
    );
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct Bound(String);

impl From<&str> for Bound {
    fn from(s: &str) -> Bound {
        Bound(s.to_owned())
    }
}

impl fmt::Display for Bound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, PartialEq, Eq)]
struct PackageWithComponent {
    package: Package,
    version: Version,
    bound: Bound,
    component: String,
}

fn parse_package_with_component(s: &str) -> Option<PackageWithComponent> {
    let package = regex!(
        r#"^- \[ \] (?P<package>[\da-zA-z][\da-zA-Z-]*?)-(?P<version>(\d+(\.\d+)*)) \((?P<bound>[^)]+)\).+?Used by: (?P<component>.+)$"#
    );
    Captures::new(package, s)
        .ok()
        .map(|cap| PackageWithComponent {
            package: cap.name("package").unwrap(),
            version: cap.try_name("version").unwrap(),
            bound: cap.try_name("bound").unwrap(),
            component: cap.name("component").unwrap(),
        })
}

#[test]
fn test_parse_header_versioned() {
    let s = "aeson-2.0.3.0 ([changelog](http://hackage.haskell.org/package/aeson-2.0.3.0/changelog)) (Adam Bergmark <adam@bergmark.nl> @bergmark, Stackage upper bounds) is out of bounds for:";
    assert_eq!(
        parse_header_versioned(s).unwrap(),
        Header::Versioned(VersionedPackage {
            package: "aeson".into(),
            version: "2.0.3.0".try_into().unwrap(),
        })
    )
}

fn parse_header_versioned(s: &str) -> Option<Header> {
    let header_versioned = regex!(
        r#"^(?P<package>[\da-zA-z][\da-zA-Z-]*?)-(?P<version>(\d+(\.\d+)*)).+?is out of bounds for:$"#
    );
    Captures::new(header_versioned, s).ok().map(|cap| {
        Header::Versioned(VersionedPackage {
            package: cap.name("package").unwrap(),
            version: cap.try_name("version").unwrap(),
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

    Captures::new(header_missing, s)
        .ok()
        .map(|cap| Header::Missing(cap.name("package").unwrap()))
}

type H = HashMap<Header, Vec<(Package, Version, Bound, String)>>;

pub fn add(build_constraints: &Path) {
    add_impl(build_constraints, || {
        io::stdin().lock().lines().flatten().collect()
    });
}

pub fn add_impl(build_constraints: &Path, lines: impl FnOnce() -> Vec<String>) -> usize {
    let mut lib_exes: H = Default::default();
    let mut tests: H = Default::default();
    let mut benches: H = Default::default();
    let mut last_header: Option<Header> = None;

    // Ignore everything until the bounds issues show up.
    let mut process_line = false;

    for line in lines() {
        if regex!(r#"^\s*$"#).is_match(&line) {
            // noop
        } else if line == "curator: Snapshot dependency graph contains errors:" {
            process_line = true;
        } else if !process_line {
            println!("[INFO] {line}");
        } else if let Some(PackageWithComponent {
            package,
            version,
            bound,
            component,
        }) = parse_package_with_component(&line)
        {
            let root = last_header.clone().unwrap();
            match &*component {
                "library" | "executable" => {
                    insert(&mut lib_exes, root, &package, &version, &bound, &component)
                }
                "benchmark" => insert(&mut benches, root, &package, &version, &bound, "benchmarks"),
                "test-suite" => insert(&mut tests, root, &package, &version, &bound, &component),
                _ => panic!("Bad component: {component}"),
            }
        } else if let Some(header_versioned) = parse_header_versioned(&line) {
            last_header = Some(header_versioned);
        } else if let Some(missing) = parse_header_missing(&line) {
            last_header = Some(missing);
        } else {
            panic!("Unhandled: {line:?}");
        }
    }

    let mut auto_lib_exes = vec![];
    let mut auto_tests = vec![];
    let mut auto_benches = vec![];

    if !lib_exes.is_empty() {
        println!("\nLIBS + EXES\n");
    }
    for (header, packages) in lib_exes {
        for (package, version, bound, component) in packages {
            let s = printer(
                "        ", &package, true, &version, &bound, &component, &header,
            );
            println!("{s}");
            auto_lib_exes.push(s);
        }
    }

    if !tests.is_empty() {
        println!("\nTESTS\n");
    }
    for (header, packages) in tests {
        for (package, version, bound, component) in packages {
            let s = printer(
                "    ", &package, false, &version, &bound, &component, &header,
            );
            println!("{s}");
            auto_tests.push(s);
        }
    }

    if !benches.is_empty() {
        println!("\nBENCHMARKS\n");
    }
    for (header, packages) in benches {
        for (package, version, bound, component) in packages {
            let s = printer(
                "    ", &package, false, &version, &bound, &component, &header,
            );
            println!("{s}");
            auto_benches.push(s);
        }
    }

    let lib_exe_count = auto_lib_exes.len();

    println!();
    println!(
        "Adding {lib_exes} libs, {tests} tests, {benches} benches to build-constraints.yaml",
        lib_exes = lib_exe_count,
        tests = auto_tests.len(),
        benches = auto_benches.len()
    );
    adder(build_constraints, auto_lib_exes, auto_tests, auto_benches);

    lib_exe_count
}

fn printer(
    indent: &str,
    package: &Package,
    lt0: bool,
    version: &Version,
    bound: &Bound,
    component: &str,
    header: &Header,
) -> String {
    let lt0 = if lt0 { " < 0" } else { "" };
    format!(
        "{indent}- {package}{lt0} # tried {package}-{version}, but its *{component}* {cause}",
        cause = match header {
            Header::Versioned(versioned) => format!(
                "requires {package} {bound}, but the snapshot contains {versioned}",
                package = versioned.package
            ),
            Header::Missing(package) => format!("requires the disabled package: {package}"),
        },
    )
}

fn insert(
    h: &mut H,
    header: Header,
    package: &Package,
    version: &Version,
    bound: &Bound,
    component: &str,
) {
    (*h.entry(header).or_insert_with(Vec::new)).push((
        package.clone(),
        version.clone(),
        bound.clone(),
        component.to_owned(),
    ));
}

fn adder(build_constraints: &Path, lib: Vec<String>, test: Vec<String>, bench: Vec<String>) {
    handle(build_constraints, true, |loc, mut lines| {
        lines.extend(match loc {
            Location::Lib => lib.clone(),
            Location::Test => test.clone(),
            Location::Bench => bench.clone(),
        });
        lines.sort();
        lines
    });
}
