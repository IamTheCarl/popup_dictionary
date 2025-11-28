use std::{env, process};

fn main() {
    #[cfg(debug_assertions)]
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    #[cfg(not(debug_assertions))]
    env_logger::init();

    let args: Vec<String> = env::args().skip(1).collect();

    let sentence: String = args
        .join("")
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();

    if sentence.starts_with("--copy") || sentence.starts_with("-c") {
        if let Err(e) = popup_dictionary::copy() {
            eprintln!("Error: {e}");
            process::exit(1);
        }
    } else if !sentence.is_empty() {
        if let Err(e) = popup_dictionary::run(&sentence) {
            eprintln!("Error: {e}");
            process::exit(1);
        }
    } else {
        eprintln!("No text provided for processing.");
        process::exit(1);
    }
}
