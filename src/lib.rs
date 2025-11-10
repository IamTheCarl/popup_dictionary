use std::error::Error;

use crate::app::run_app;
//use crate::dictionary::Dictionary;
//use crate::tokenizer::{ParsedWord, tokenize};

mod app;
//mod dictionary;
mod plugin;
mod plugins;
//mod tokenizer;

pub fn run(sentence: &str) -> Result<(), Box<dyn Error>> {
    /*println!("loading dict");
    let db_path: PathBuf = match dirs::config_dir() {
        Some(path) => path.join("popup_dictionary/db"),
        None => Err("No valid config path found in environment variables.")?,
    };
    let dictionary: Dictionary = Dictionary::load_dictionary(&db_path)?;

    println!("tokenizing");
    let words: Vec<ParsedWord> = tokenize(&sentence.to_string(), &dictionary)?;*/

    println!("displaying");
    run_app(sentence)?;

    Ok(())
}
