use crate::prelude::*;

use crate::build_constraints::{self, Maintenance};

pub fn maintainers(build_constraints: &Path) {
    let bc = build_constraints::parse(build_constraints);
    for maintainer in bc.maintainers() {
        if let Maintenance::Maintainer(maintainer) = maintainer {
            if maintainer.github_users().next().is_none() {
                println!("{maintainer}: Missing github handle")
            }
        }
    }
}
