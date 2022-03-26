use crate::prelude::*;

use crate::build_constraints;

pub fn multiple(build_constraints: &Path) {
    let bc = build_constraints::parse(build_constraints);
    let m = build_constraints::transpose(bc.packages);
    for (package, maintainers) in m {
        if maintainers.len() >= 2 {
            let v: Vec<String> = maintainers.into_iter().map(|m| m.0).collect();
            println!("{package}: {}", v.join(", "),);
        }
    }
}
