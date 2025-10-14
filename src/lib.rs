use std::error::Error;
use std::path::PathBuf;

use crate::app::run_app;
use crate::dictionary::Dictionary;
use crate::tokenizer::{tokenize, ParsedWord};

mod app;
mod dictionary;
mod tokenizer;

pub fn run(query: &String) -> Result<(), Box<dyn Error>> {
    println!("loading dict");
    let db_path: PathBuf = match dirs::config_dir() {
        Some(path) => path.join("popup_dictionary/db"),
        None => Err("No valid config path found in environment variables.")?,
    };
    let dictionary: Dictionary = Dictionary::load_dictionary(&db_path)?;

    println!("tokenizing");
    let words: Vec<ParsedWord> = tokenize(&query, &dictionary)?;

    println!("displaying");
    run_app(&words, &dictionary)?;

    Ok(())
}
