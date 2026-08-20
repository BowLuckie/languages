use std::{
    error::Error,
    fs::{self},
};

use fl::parser::parse;

fn main() -> anyhow::Result<(), Box<dyn Error>> {
    let source = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/test.fl"))?;
    parse(source.as_str())?;
    Ok(())
}
