use clap::Parser;
use image::DynamicImage;
use image::ImageReader;
use std::io::Cursor;
use std::io::Read;
use std::path::PathBuf;
use std::process::ExitCode;

/// Simple Popup dictionary
#[derive(Parser, Debug)]
#[command(name = "popup dictionary", version, about, long_about = None, arg_required_else_help(true))]
struct Args {
    #[clap(flatten)]
    action: Action,

    /// Initial plugin to load. Available: "kihon", "jotoba"
    #[arg(long = "initial-plugin", value_name = "PLUGIN")]
    initial_plugin: Option<String>,
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

    /// Use OCR mode. Reads image from path if provided, otherwise takes image data from stdin
    #[arg(short = 'o', long = "ocr", value_name = "PATH")]
    ocr: Option<Option<PathBuf>>,
}

fn main() -> ExitCode {
    #[cfg(debug_assertions)]
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    #[cfg(not(debug_assertions))]
    env_logger::init();

    let cli: Args = Args::parse();

    if let Some(text) = &cli.action.text {
        if let Err(e) = popup_dictionary::run(&text, &cli.initial_plugin) {
            eprintln!("Error: {e}");
            return ExitCode::FAILURE;
        }
    } else if cli.action.primary {
        if let Err(e) = popup_dictionary::primary(&cli.initial_plugin) {
            eprintln!("Error: {e}");
            return ExitCode::FAILURE;
        }
    } else if cli.action.secondary {
        if let Err(e) = popup_dictionary::secondary(&cli.initial_plugin) {
            eprintln!("Error: {e}");
            return ExitCode::FAILURE;
        }
    } else if cli.action.clipboard {
        if let Err(e) = popup_dictionary::clipboard(&cli.initial_plugin) {
            eprintln!("Error: {e}");
            return ExitCode::FAILURE;
        }
    } else if cli.action.copy {
        if let Err(e) = popup_dictionary::copy(&cli.initial_plugin) {
            eprintln!("Error: {e}");
            return ExitCode::FAILURE;
        }
    } else if let Some(ocr_path) = cli.action.ocr {
        match get_image_for_ocr(ocr_path) {
            Ok(image) => {
                if let Err(e) = popup_dictionary::ocr(image, &cli.initial_plugin) {
                    eprintln!("Error: {e}");
                    return ExitCode::FAILURE;
                }
            }
            Err(e) => {
                eprintln!("Error: OCR mode requires path or image data to be provided.\n{e}");
                return ExitCode::FAILURE;
            }
        }
    }

    ExitCode::SUCCESS
}

fn get_image_for_ocr(ocr_arg: Option<PathBuf>) -> Result<DynamicImage, Box<dyn std::error::Error>> {
    match ocr_arg {
        Some(path) => Ok(image::open(path)?),
        None => {
            let mut buffer = Vec::new();
            std::io::stdin().read_to_end(&mut buffer)?;

            let img: DynamicImage = ImageReader::new(Cursor::new(buffer))
                .with_guessed_format()?
                .decode()?;

            Ok(img)
        }
    }
}
