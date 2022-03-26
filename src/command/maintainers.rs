use crate::prelude::*;

use crate::build_constraints;

pub fn maintainers(build_constraints: &Path) {
    let bc = build_constraints::parse(build_constraints);
    for maintainer in bc.maintainers() {
        if maintainer.github_users().next().is_none() {
            println!("{maintainer}: Missing github handle")
        }
    }
}
