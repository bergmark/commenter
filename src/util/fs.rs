use anyhow::{Context, Result};
use std::fmt;
use std::fs::File;
pub use std::fs::ReadDir;
use std::io::{BufRead, BufReader, Lines};
use std::path::Path;

pub(crate) use std::fs::metadata;

pub(crate) fn read_lines<P>(filename: P) -> Lines<BufReader<File>>
where
    P: AsRef<Path> + fmt::Debug + Clone,
{
    let filen = filename.clone();
    let file =
        File::open(filename).unwrap_or_else(|e| panic!("Could not open {filen:?}, error: {e}"));
    BufReader::new(file).lines()
}

pub fn read_dir<P: AsRef<Path> + fmt::Debug>(path: P) -> Result<ReadDir> {
    std::fs::read_dir(&path).with_context(|| format!("Error reading directory {path:?}"))
}
