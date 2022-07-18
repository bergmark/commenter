use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader, Lines};
use std::path::Path;

pub(crate) fn read_lines<P>(filename: P) -> Lines<BufReader<File>>
where
    P: AsRef<Path> + fmt::Debug + Clone,
{
    let filen = filename.clone();
    let file =
        File::open(filename).unwrap_or_else(|e| panic!("Could not open {filen:?}, error: {e}"));
    BufReader::new(file).lines()
}
