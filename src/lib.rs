use std::error::Error;

use crate::app::run_app;
use crate::parser::{ParsedWord, tokenize};

mod app;
mod parser;

const DICT_DATA: &[u8] = include_bytes!("./dictionaries/system.dic");

pub fn run(query: &String) -> Result<(), Box<dyn Error>> {
    let words: Vec<ParsedWord> = tokenize(&query)?;

    run_app(&words)?;

    Ok(())
}
