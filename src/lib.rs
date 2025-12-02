use arboard::{Clipboard, GetExtLinux};
use enigo::{Enigo, Keyboard};
use std::error::Error;

use crate::app::run_app;

mod app;
mod plugin;
mod plugins;
mod window_helper;

pub fn run(sentence: &str) -> Result<(), Box<dyn Error>> {
    run_app(sentence)?;

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
