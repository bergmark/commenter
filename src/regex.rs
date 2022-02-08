use lazy_regex::Captures;

pub fn cap_into<'a, A: From<&'a str>>(cap: &Captures<'a>, name: &'static str) -> A {
    cap.name(name).unwrap().as_str().into()
}
