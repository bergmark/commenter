use std::error::Error;
use std::fs::File;
use std::io::BufReader;

use serde::Deserialize;

use crate::prelude::*;

pub fn yaml_from_file<A, P: AsRef<Path>>(path: P) -> Result<A, Box<dyn Error>>
where
    A: for<'de> Deserialize<'de>,
{
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let u = serde_yaml::from_reader(reader)?;
    Ok(u)
}
