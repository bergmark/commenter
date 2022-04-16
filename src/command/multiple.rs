use crate::prelude::*;

use crate::build_constraints;

pub fn multiple(build_constraints: &Path) {
    let bc = build_constraints::parse(build_constraints).by_package();
    for (package, bc) in bc.packages {
        let maintainers: Vec<_> = bc
            .maintainers
            .into_iter()
            .filter_map(|m| m.maintainer().map(|m| m.to_string()))
            .collect();
        if maintainers.len() >= 2 {
            println!("{package}: {}", maintainers.join(", "));
        }
    }
}
