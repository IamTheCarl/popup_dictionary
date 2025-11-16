use std::{env, process};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let sentence: String = args
        .join("")
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    if !sentence.is_empty() {
        if let Err(e) = popup_dictionary::run(&sentence) {
            eprintln!("Error: {e}");
            process::exit(1);
        }
    } else {
        eprintln!("No text provided for processing.");
        process::exit(1);
    }
}
