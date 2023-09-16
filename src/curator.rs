use std::process::{Command, Output};

pub fn update() {
    let res = call_curator(&["update"]);

    for line in lines(res.stderr) {
        println!("[curator update] {line}");
    }
}

pub fn constraints(target: &str) {
    let res = call_curator(&["constraints", &*format!("--target={target}")]);

    for line in lines(res.stderr) {
        println!("[curator constraints] {line}");
    }
}

pub fn snapshot_incomplete(target: &str) {
    let res = call_curator(&["snapshot-incomplete", &*format!("--target={target}")]);

    for line in lines(res.stderr) {
        println!("[curator snapshot] {line}");
    }
}

pub fn snapshot() {
    let res = call_curator(&["snapshot"]);

    for line in lines(res.stderr) {
        println!("[curator snapshot] {line}");
    }
}

pub fn stack(ghc_version: &str) -> Vec<String> {
    let output = Command::new("stack")
        .args([
            "--resolver",
            &*format!("ghc-{ghc_version}"),
            "exec",
            "curator",
            "check-snapshot",
        ])
        .output()
        .expect("Could not find stack in PATH");

    lines(output.stderr)
}

fn call_curator(args: &[&str]) -> Output {
    Command::new("curator")
        .args(args)
        .output()
        .expect("Could not find curator in PATH")
}

fn lines(x: Vec<u8>) -> Vec<String> {
    String::from_utf8(x)
        .unwrap()
        .trim()
        .to_owned()
        .lines()
        .map(|s| s.to_owned())
        .collect()
}
