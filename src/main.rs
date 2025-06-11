use std::{env, process};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.len() > 0 {
        let query: String = args.join("");
        if let Err(e) = popup_dictionary::run(&query) {
            eprintln!("Error: {e}");
            process::exit(1);
        }
    } else {
        eprintln!("Error: No text provided.");
        process::exit(1);
    }
}
