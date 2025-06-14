use std::error::Error;

use crate::app::run_app;
use crate::dictionary::Dictionary;
use crate::tokenizer::{ParsedWord, tokenize};

mod app;
mod dictionary;
mod tokenizer;

pub fn run(query: &String) -> Result<(), Box<dyn Error>> {
    println!("tokenizing");
    let words: Vec<ParsedWord> = tokenize(&query)?;

    println!("loading dict");
    let dictionary: Dictionary =
        Dictionary::load_dictionary("./dictionaries/jmdict-simplified.db")?;

    println!("displaying");
    run_app(&words, &dictionary)?;

    Ok(())
}
