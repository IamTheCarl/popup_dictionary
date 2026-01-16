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

    /// Try to open the window at the mouse cursor. Unlikely to work on wayland
    #[arg(short = 'm', long = "at-mouse")]
    open_at_cursor: bool,

    /// Display input text in a text-box instead of in one line
    #[arg(short = 'f', long = "full-text")]
    wrapped: bool,

    /// Initial window width in pixels. Default: 450
    #[arg(long = "width", value_name = "PIXELS")]
    initial_width: Option<u16>,

    /// Initial window height in pixels. Default: 450
    #[arg(long = "height", value_name = "PIXELS")]
    initial_height: Option<u16>,
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

    /*
    /// Copy text by simulating Ctrl+C and pass from clipboard as input text
    #[arg(short = 'c', long = "copy")]
    copy: bool,

    /// Watch clipboard for newly copied text
    #[arg(short = 'w', long = "watch")]
    watch: bool,
    */
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

    let config: popup_dictionary::app::Config = popup_dictionary::app::Config {
        initial_plugin: cli.initial_plugin,
        open_at_cursor: cli.open_at_cursor,
        wrapped: cli.wrapped,
        initial_width: cli.initial_width.unwrap_or(450),
        initial_height: cli.initial_height.unwrap_or(450),
    };

    if let Some(text) = &cli.action.text {
        if let Err(e) = popup_dictionary::run(&text, config) {
            eprintln!("Error: {e}");
            return ExitCode::FAILURE;
        }
    } else if cli.action.primary {
        if let Err(e) = popup_dictionary::primary(config) {
            eprintln!("Error: {e}");
            return ExitCode::FAILURE;
        }
    } else if cli.action.secondary {
        if let Err(e) = popup_dictionary::secondary(config) {
            eprintln!("Error: {e}");
            return ExitCode::FAILURE;
        }
    } else if cli.action.clipboard {
        if let Err(e) = popup_dictionary::clipboard(config) {
            eprintln!("Error: {e}");
            return ExitCode::FAILURE;
        }
    /*
    } else if cli.action.copy {
        if let Err(e) = popup_dictionary::copy(&cli.initial_plugin) {
            eprintln!("Error: {e}");
            return ExitCode::FAILURE;
        }

    } else if cli.action.watch {
        if let Err(e) = popup_dictionary::watch(config) {
            eprintln!("Error: {e}");
            return ExitCode::FAILURE;
        }
    */
    } else if let Some(ocr_path) = cli.action.ocr {
        match get_image_for_ocr(ocr_path) {
            Ok(image) => {
                if let Err(e) = popup_dictionary::ocr(image, config) {
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
