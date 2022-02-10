pub mod add;
pub mod affected;
pub mod diff_snapshot;
pub mod disabled;
pub mod multiple;
pub mod outdated;

use crate::handle::{handle, Location};

pub fn clear() {
    handle(true, |loc, _lines| match loc {
        // Add empty array to keep yaml valid
        Location::Lib => vec!["        []".to_owned()],
        Location::Test | Location::Bench => vec![],
    });
}
