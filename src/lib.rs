use std::error::Error;

use crate::app::run_app;
use crate::dictionary::Dictionary;
use crate::tokenizer::{ParsedWord, tokenize};

mod app;
mod dictionary;
mod tokenizer;

pub fn run(query: &String) -> Result<(), Box<dyn Error>> {
    let words: Vec<ParsedWord> = tokenize(&query)?;

    let dictionary: Dictionary =
        Dictionary::load_dictionary("./dictionaries/jmdict-simplified.db")?;

    run_app(&words, &dictionary)?;

    Ok(())
}
