use crate::prelude::*;

pub mod add;
pub mod add_loop;
pub mod affected;
pub mod diff_snapshot;
pub mod disabled;
pub mod maintainers;
pub mod multiple;
pub mod not_present;
pub mod outdated;
pub mod package_info;

use crate::handle::{handle, Location};

pub fn clear(build_constraints: &Path) {
    handle(build_constraints, true, |loc, _lines| match loc {
        // Add empty array to keep yaml valid
        Location::Lib => vec!["        []".to_owned()],
        Location::Test | Location::Bench => vec![],
    });
}
