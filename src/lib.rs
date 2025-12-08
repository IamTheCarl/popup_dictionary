use arboard::{Clipboard, GetExtLinux};
use enigo::{Enigo, Keyboard};
use image::DynamicImage;
use std::error::Error;
use std::path::PathBuf;
use tesseract_rs::TesseractAPI;

use crate::app::run_app;

mod app;
mod plugin;
mod plugins;
mod window_helper;

pub fn run(sentence: &str) -> Result<(), Box<dyn Error>> {
    let sentence: String = sentence.chars().filter(|c| !c.is_whitespace()).collect();

    if sentence.is_empty() {
        return Err(Box::from("Input text must be at least one character."));
    }

    run_app(&sentence)?;

    Ok(())
}

pub fn primary() -> Result<(), Box<dyn Error>> {
    let mut clipboard: Clipboard = Clipboard::new()?;
    let sentence: String = clipboard
        .get()
        .clipboard(arboard::LinuxClipboardKind::Primary)
        .text()?;
    run(&sentence)
}

pub fn secondary() -> Result<(), Box<dyn Error>> {
    let mut clipboard: Clipboard = Clipboard::new()?;
    let sentence: String = clipboard
        .get()
        .clipboard(arboard::LinuxClipboardKind::Secondary)
        .text()?;
    run(&sentence)
}

pub fn clipboard() -> Result<(), Box<dyn Error>> {
    let mut clipboard: Clipboard = Clipboard::new()?;
    let sentence: String = clipboard.get().text()?;
    run(&sentence)
}

pub fn copy() -> Result<(), Box<dyn Error>> {
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

    clipboard()
}

pub fn ocr(image: DynamicImage) -> Result<(), Box<dyn Error>> {
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

    run(&sentence)
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
