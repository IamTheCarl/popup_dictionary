use clap::Parser;
use std::{env, process};

/// Simple Popup dictionary
#[derive(Parser, Debug)]
#[command(name = "popup dictionary", version, about, long_about = None, arg_required_else_help(true))]
struct Args {
    #[clap(flatten)]
    action: Action,
}

#[derive(clap::Args, Debug)]
#[group(required = true, multiple = false)]
struct Action {
    /// Provide input text manually
    #[arg(short = 't', long = "text", value_name = "STRING")]
    text: Option<String>,

    /// Get input text from primary clipboard/selection (linux)
    #[arg(short = 'p', long = "primary")]
    primary: bool,

    /// Get input text from secondary clipboard/selection (linux)
    #[arg(short = 's', long = "secondary")]
    secondary: bool,

    /// Get input text from clipboard
    #[arg(short = 'b', long = "clipboard")]
    clipboard: bool,

    /// Copy text by simulating Ctrl+C and pass from clipboard as input text
    #[arg(short = 'c', long = "copy")]
    copy: bool,
}

fn main() {
    #[cfg(debug_assertions)]
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    #[cfg(not(debug_assertions))]
    env_logger::init();

    let cli: Args = Args::parse();

    if let Some(text) = &cli.action.text {
        let sentence: String = text.chars().filter(|c| !c.is_whitespace()).collect();

        if let Err(e) = popup_dictionary::run(&sentence) {
            eprintln!("Error: {e}");
            process::exit(1);
        }
    } else if cli.action.primary {
        if let Err(e) = popup_dictionary::primary() {
            eprintln!("Error: {e}");
            process::exit(1);
        }
    } else if cli.action.secondary {
        if let Err(e) = popup_dictionary::secondary() {
            eprintln!("Error: {e}");
            process::exit(1);
        }
    } else if cli.action.clipboard {
        if let Err(e) = popup_dictionary::clipboard() {
            eprintln!("Error: {e}");
            process::exit(1);
        }
    } else if cli.action.copy {
        if let Err(e) = popup_dictionary::copy() {
            eprintln!("Error: {e}");
            process::exit(1);
        }
    }
}
