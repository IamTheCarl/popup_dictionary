<h2 align="center">Popup Dictionary [WIP]</h2>

This application is a WIP pop-up dictionary (currently for Japanese->English) that works outside your browser.  

It is inspired by tools like [Yomitan](https://github.com/yomidevs/yomitan), an incredibly useful browser extension that lets you look-up the definition of words with the press of a button.
Since Yomitan only works in the browser, I decided to make a pop-up dictionary application that works outside the browser, using methods like OCR or the system clipboard for looking up text.  
The main difference to something like Yomitan is that the look-up is not restricted to a single word, but you can copy/OCR a whole sentence or even full blocks of text and look-up the individual words.


## Installation
> [!NOTE]\
> Currently, the main focus of development is on Linux (x11 and wayland). Basic Windows support should already be there, however some features may or may not fully work until I get to it.
### Linux
[WIP]

### Windows
[WIP]

## Usage
[WIP]

## Building
This project is built in 🔥blazingly-fast, memory-safe🔥 Rust. Building and running it from source should be relatively simple using the Rust toolchain/Cargo. You can find installation instructions here [rustup.rs](https://rustup.rs/).

#### Dependencies:
[WIP]

#### Steps:
1. Clone the repository:
   ```sh
   git clone https://github.com/jasmine-blush/popup_dictionary.git
   cd popup_dictionary
   ```
2. Build it:
   ```sh
   cargo build
   ```
   The compiled binary/executable will be inside the ``target`` directory.
3. Or run it directly:
   ```sh
   cargo run
   ```
   In order to pass arguments to the popup dictionary when running it like this, use:
   ```sh
   cargo run -- [arguments]
   ```
You can use ``--release`` for an optimized release build.
There are two optional feature-flags ``wayland-support`` and ``hyprland-support`` that enable better wayland and hyprland support respectively. You can use them like this:
   ```sh
   cargo build --features hyprland-support
   ```

All in all, a command could look something like this:
   ```sh
   cargo run --release --features wayland-support -- --watch --tray
   ```
