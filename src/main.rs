use std::{
    env,
    process::{self, Command},
};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let sentence: String = args
        .join("")
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();

    if sentence.starts_with("--copy") || sentence.starts_with("-c") {
        use arboard::Clipboard;
        use enigo::{Enigo, Keyboard};

        // send Ctrl+C (twice as workaround for not always registering)
        let mut enigo = Enigo::new(&enigo::Settings::default()).unwrap();
        enigo.set_delay(100);
        enigo
            .key(enigo::Key::Control, enigo::Direction::Press)
            .unwrap();
        enigo
            .key(enigo::Key::Unicode('c'), enigo::Direction::Click)
            .unwrap();
        std::thread::sleep(core::time::Duration::from_millis(100));
        enigo
            .key(enigo::Key::Control, enigo::Direction::Release)
            .unwrap();
        std::thread::sleep(core::time::Duration::from_millis(100));
        enigo
            .key(enigo::Key::Control, enigo::Direction::Press)
            .unwrap();
        enigo
            .key(enigo::Key::Unicode('c'), enigo::Direction::Click)
            .unwrap();
        std::thread::sleep(core::time::Duration::from_millis(100));
        enigo
            .key(enigo::Key::Control, enigo::Direction::Release)
            .unwrap();
        std::thread::sleep(core::time::Duration::from_millis(100));

        println!("creating clipboard");
        if let Ok(mut clipboard) = Clipboard::new() {
            println!("getting text");
            if let Ok(sentence) = clipboard.get().text() {
                println!("Clipboard content: {}", sentence);
                if let Err(e) = popup_dictionary::run(&sentence) {
                    eprintln!("Error: {e}");
                    process::exit(1);
                }
            }
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
