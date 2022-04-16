use crate::prelude::*;

use regex::{self, Regex};

pub struct Captures<'a>(regex::Captures<'a>);

impl<'a> Captures<'a> {
    pub fn new<'b: 'a>(regex: &Regex, s: &'b str) -> Result<Captures<'a>, anyhow::Error> {
        let c = regex.captures(s).with_context(|| "{r} did not match {s}")?;
        Ok(Captures(c))
    }

    pub fn get<A: From<&'a str>>(&self, n: usize) -> Result<A, anyhow::Error> {
        self.0
            .get(n)
            .with_context(|| format!("Could not find capture group {n}"))
            .map(|m| m.as_str().into())
    }

    pub fn try_get<A>(&self, n: usize) -> Result<A, anyhow::Error>
    where
        A: TryFrom<&'a str>,
        A::Error: Send + Sync + std::error::Error,
    {
        let value = self
            .0
            .get(n)
            .with_context(|| format!("Could not find capture group {n}"))?;
        let value = value.as_str();

        match value.try_into() {
            Err(e) => Err(anyhow!(
                "Could not parse {value} in capture group {n}, error: {e}"
            )),
            Ok(r) => Ok(r),
        }
    }

    pub fn name<A: From<&'a str>>(&self, name: &'static str) -> Result<A, anyhow::Error> {
        self.0
            .name(name)
            .with_context(|| format!("Could not find capture group {name}"))
            .map(|m| m.as_str().into())
    }

    pub fn try_name<A>(&self, name: &'static str) -> Result<A, anyhow::Error>
    where
        A: TryFrom<&'a str>,
        A::Error: Send + Sync + std::error::Error,
    {
        let value = self
            .0
            .name(name)
            .with_context(|| format!("Could not find capture group {name}"))?;
        let value = value.as_str();

        match value.try_into() {
            Err(e) => Err(anyhow!(
                "Could not parse {value} in capture group {name}, error: {e}"
            )),
            Ok(r) => Ok(r),
        }
    }
}
