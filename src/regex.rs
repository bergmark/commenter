use lazy_regex::Captures;

pub fn cap_into<'a, A: From<&'a str>>(cap: &Captures<'a>, name: &'static str) -> A {
    cap_into_opt(cap, name).unwrap()
}

pub fn cap_into_n<'a, A: From<&'a str>>(cap: &Captures<'a>, n: usize) -> A {
    cap_into_opt_n(cap, n).unwrap()
}

pub fn cap_try_into<'a, A: TryFrom<&'a str>>(cap: &Captures<'a>, name: &'static str) -> A {
    cap_try_into_opt(cap, name).unwrap()
}

pub fn cap_try_into_n<'a, A: TryFrom<&'a str>>(cap: &Captures<'a>, n: usize) -> A {
    cap_try_into_opt_n(cap, n).unwrap()
}

pub fn cap_into_opt<'a, A: From<&'a str>>(cap: &Captures<'a>, name: &'static str) -> Option<A> {
    cap.name(name).map(|m| m.as_str().into())
}

pub fn cap_into_opt_n<'a, A: From<&'a str>>(cap: &Captures<'a>, n: usize) -> Option<A> {
    cap.get(n).map(|m| m.as_str().into())
}

pub fn cap_try_into_opt<'a, A: TryFrom<&'a str>>(
    cap: &Captures<'a>,
    name: &'static str,
) -> Option<A> {
    cap.name(name).and_then(|m| m.as_str().try_into().ok())
}

pub fn cap_try_into_opt_n<'a, A: TryFrom<&'a str>>(cap: &Captures<'a>, n: usize) -> Option<A> {
    cap.get(n).and_then(|m| m.as_str().try_into().ok())
}
