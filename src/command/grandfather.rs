use crate::prelude::*;

use crate::command::disabled::{self, DisabledTransitively, M};
use crate::handle::{self, Location};

pub fn grandfather(build_constraints: &Path) {
    let mut disabled_transitively: Vec<DisabledTransitively> = vec![];
    handle::handle(build_constraints, false, |loc, lines| {
        match loc {
            Location::Lib => disabled_transitively.extend(
                lines
                    .into_iter()
                    .filter_map(|line| disabled::parse_disabled_transitviely(&line))
                    .collect::<Vec<_>>(),
            ),
            Location::Test | Location::Bench => (),
        }
        vec![]
    });

    let mut disabled: M = BTreeMap::new();

    for DisabledTransitively { child, parent } in disabled_transitively {
        disabled
            .entry(child.package.clone())
            .or_insert_with(|| (vec![], None));
        let t = disabled.entry(parent).or_insert_with(|| (vec![], None));
        t.0.push(child);
    }

    let bc = crate::build_constraints::parse(build_constraints).by_package();

    for (parent, (children, _)) in disabled.into_iter() {
        if children.is_empty() {
            continue;
        }
        if bc.package(&parent).is_none() {
            println!("        - {parent}");
        }
    }
}
