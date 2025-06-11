use crate::parser::{ParsedWord, parse_words};
use std::error::Error;

use crate::app::run_app;

mod app;
mod parser;

pub fn run(query: &String) -> Result<(), Box<dyn Error>> {
    let words: Vec<ParsedWord> = parse_words(&query)?;

    run_app(&words)?;

    Ok(())
}
