use crate::prelude::*;

use crate::build_constraints;

pub fn multiple(build_constraints: &Path) {
    let bc = build_constraints::parse(build_constraints).by_package();
    for (package, bc) in bc.packages {
        if bc.maintainers.len() >= 2 {
            let v: Vec<String> = bc.maintainers.into_iter().map(|m| m.0).collect();
            println!("{package}: {}", v.join(", "),);
        }
    }
}
