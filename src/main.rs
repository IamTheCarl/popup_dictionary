use std::{env, process};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let query: String = args.join("").replace(" ", "");
    if !query.is_empty() {
        if let Err(e) = popup_dictionary::run(&query) {
            eprintln!("Error: {e}");
            process::exit(1);
        }
    } else {
        eprintln!("No text provided for processing.");
        process::exit(1);
    }
}
