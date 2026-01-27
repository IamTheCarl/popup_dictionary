use arboard::Clipboard;
#[cfg(target_os = "linux")]
use arboard::GetExtLinux;
use enigo::{Enigo, Keyboard};
use image::DynamicImage;
use regex::Regex;
use std::error::Error;
use std::path::PathBuf;
use std::sync::OnceLock;

use crate::app::run_app;
use crate::tesseract::{check_tesseract, ocr_image};

pub mod app;
mod plugin;
mod plugins;
mod tesseract;
mod window_helper;

pub fn run(sentence: &str, config: app::Config) -> Result<(), Box<dyn Error>> {
    let sentence: String = sentence.chars().filter(|c| !c.is_whitespace()).collect();

    if sentence.is_empty() {
        return Err(Box::from("Input text must be at least one character."));
    }

    if !contains_japanese(&sentence) {
        return Err(Box::from("Input text must contain japanese text."));
    }

    run_app(&sentence, config)?;

    Ok(())
}

fn contains_japanese(text: &str) -> bool {
    static RE: OnceLock<Regex> = OnceLock::new();

    let re = RE.get_or_init(|| {
        Regex::new(concat!(
            r"[",
            r"\p{scx=Hiragana}",
            r"\p{scx=Katakana}",
            r"\p{scx=Han}", // Kanji, Hanzi, Hanja
            r"]"
        ))
        .expect("Regex compilation failed")
    });

    re.is_match(text)
}

#[cfg(target_os = "linux")]
pub fn primary(config: app::Config) -> Result<(), Box<dyn Error>> {
    let mut clipboard: Clipboard = Clipboard::new()?;
    let sentence: String = clipboard
        .get()
        .clipboard(arboard::LinuxClipboardKind::Primary)
        .text()?;
    run(&sentence, config)
}

#[cfg(target_os = "linux")]
pub fn secondary(config: app::Config) -> Result<(), Box<dyn Error>> {
    let mut clipboard: Clipboard = Clipboard::new()?;
    let sentence: String = clipboard
        .get()
        .clipboard(arboard::LinuxClipboardKind::Secondary)
        .text()?;
    run(&sentence, config)
}

pub fn clipboard(config: app::Config) -> Result<(), Box<dyn Error>> {
    let mut clipboard: Clipboard = Clipboard::new()?;
    let sentence: String = clipboard.get().text()?;
    run(&sentence, config)
}

/*
pub fn copy(initial_plugin: &Option<String>) -> Result<(), Box<dyn Error>> {
    // send Ctrl+C (twice as workaround for not always registering)
    let mut enigo: Enigo = Enigo::new(&enigo::Settings::default())?;
    enigo.set_delay(100);
    enigo.key(enigo::Key::Control, enigo::Direction::Press)?;
    enigo.key(enigo::Key::Unicode('c'), enigo::Direction::Click)?;
    std::thread::sleep(core::time::Duration::from_millis(100));
    enigo.key(enigo::Key::Control, enigo::Direction::Release)?;
    std::thread::sleep(core::time::Duration::from_millis(100));
    enigo.key(enigo::Key::Control, enigo::Direction::Press)?;
    enigo.key(enigo::Key::Unicode('c'), enigo::Direction::Click)?;
    std::thread::sleep(core::time::Duration::from_millis(100));
    enigo.key(enigo::Key::Control, enigo::Direction::Release)?;
    std::thread::sleep(core::time::Duration::from_millis(100));

    clipboard(initial_plugin)
}

*/

pub fn watch(config: app::Config) -> Result<(), Box<dyn Error>> {
    let mut clipboard: Clipboard = Clipboard::new()?;
    let mut initial_content: String = clipboard.get().text()?;

    loop {
        std::thread::sleep(std::time::Duration::from_millis(200));
        let sentence: String = clipboard.get().text()?;
        if initial_content != sentence {
            println!("initial: {}, new: {}", initial_content, sentence);
            if let Err(e) = run(&sentence, config.clone()) {
                eprintln!("Error: {e}");
            }
            initial_content = clipboard.get().text()?;
        }
    }

    Ok(())
}

pub fn ocr(image: DynamicImage, config: app::Config) -> Result<(), Box<dyn Error>> {
    if let Err(e) = check_tesseract() {
        eprintln!(
            "Error: Tesseract could not be found. Make sure you have Tesseract installed if you want to use ocr."
        );
        eprint!("Tesseract Error: ");
        return Err(Box::from(e));
    }

    let mut image_data = Vec::new();
    image.write_to(
        &mut std::io::Cursor::new(&mut image_data),
        image::ImageFormat::Png,
    )?;

    let sentence = ocr_image(&image_data)?;

    run(&sentence, config)

    /*
    let image = image.to_rgb8();
    let width: i32 = image.width() as i32;
    let height: i32 = image.height() as i32;
    const BYTES_PER_PIXEL: i32 = 3;
    let bytes_per_line: i32 = width * BYTES_PER_PIXEL;
    let image_data: &[u8] = &image.into_raw();

    let tessdata_dir: PathBuf = get_tessdata_dir();
    let tessdata_dir: &str = tessdata_dir.to_str().unwrap();
    let tess: TesseractAPI = TesseractAPI::new();

    // try horizontal ocr
    tess.init(tessdata_dir, "jpn")?;
    tess.set_image(image_data, width, height, BYTES_PER_PIXEL, bytes_per_line)?;
    let mut sentence: String = tess.get_utf8_text()?;
    let horizontal_conf: i32 = tess.mean_text_conf()?;

    // try vertical ocr
    tess.clear()?;
    tess.init(tessdata_dir, "jpn_vert")?;
    tess.set_page_seg_mode(tesseract_rs::TessPageSegMode::PSM_SINGLE_BLOCK_VERT_TEXT)?;
    tess.set_image(image_data, width, height, BYTES_PER_PIXEL, bytes_per_line)?;

    // compare confidences
    println!(
        "horz: {}, vert: {}",
        tess.mean_text_conf()?,
        horizontal_conf
    );
    if tess.mean_text_conf()? > horizontal_conf {
        sentence = tess.get_utf8_text()?;
    }

    tess.end()?;

    run(&sentence, initial_plugin)*/
}

// from tesseract-rs docs
fn get_tessdata_dir() -> PathBuf {
    match std::env::var("TESSDATA_PREFIX") {
        Ok(dir) => {
            let path = PathBuf::from(dir);
            println!("Using TESSDATA_PREFIX directory: {:?}", path);
            path
        }
        Err(_) => {
            let default_dir = get_default_tessdata_dir();
            println!(
                "TESSDATA_PREFIX not set, using default directory: {:?}",
                default_dir
            );
            default_dir
        }
    }
}

// from tesseract-rs docs
fn get_default_tessdata_dir() -> PathBuf {
    if cfg!(target_os = "macos") {
        let home_dir = std::env::var("HOME").expect("HOME environment variable not set");
        PathBuf::from(home_dir)
            .join("Library")
            .join("Application Support")
            .join("tesseract-rs")
            .join("tessdata")
    } else if cfg!(target_os = "linux") {
        let home_dir = std::env::var("HOME").expect("HOME environment variable not set");
        PathBuf::from(home_dir)
            .join(".tesseract-rs")
            .join("tessdata")
    } else if cfg!(target_os = "windows") {
        PathBuf::from(std::env::var("APPDATA").expect("APPDATA environment variable not set"))
            .join("tesseract-rs")
            .join("tessdata")
    } else {
        panic!("Unsupported operating system");
    }
}
