use crate::build_constraints;

pub fn multiple() {
    let bc = build_constraints::parse("build-constraints.yaml");
    let m = build_constraints::transpose(bc);
    for (package, maintainers) in m {
        if maintainers.len() >= 2 {
            let v: Vec<String> = maintainers.into_iter().map(|m| m.0).collect();
            println!("{package}: {}", v.join(", "),);
        }
    }
}
