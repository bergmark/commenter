use crate::prelude::*;

use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;

pub fn yaml_from_file<A>(path: &Path) -> Result<A, anyhow::Error>
where
    A: for<'de> Deserialize<'de>,
{
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let u = serde_yaml::from_reader(reader)?;
    Ok(u)
}
