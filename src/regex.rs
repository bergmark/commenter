use lazy_regex::Captures;

pub fn cap_into<'a, A: From<&'a str>>(cap: &Captures<'a>, name: &'static str) -> A {
    cap_into_opt(cap, name).unwrap()
}

pub fn cap_into_opt<'a, A: From<&'a str>>(cap: &Captures<'a>, name: &'static str) -> Option<A> {
    cap.name(name).map(|m| m.as_str().into())
}
