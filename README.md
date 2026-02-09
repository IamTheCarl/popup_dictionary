<h2 align="center">Popup Dictionary [WIP]</h2>

This application is a WIP **pop-up dictionary** (currently for Japanese->English) that works outside your browser.  

It is inspired by tools like [Yomitan](https://github.com/yomidevs/yomitan), an incredibly useful browser extension that lets you look-up the definition of words with the press of a button.
Since Yomitan only works in the browser, I decided to make a pop-up dictionary application that can be used outside the browser, utilizing methods like **OCR or the system clipboard** for looking up text.  
The main difference to something like Yomitan is that the look-up is not restricted to a single word, but you can copy/OCR a whole sentence or even full blocks of text and look-up the individual words.


## Installation
> [!NOTE]\
> Currently, the main focus of development is on **Linux** (x11 and wayland). Basic **Windows** support should already be there, however some features may or may not fully work until I get to it.
### Linux
[WIP]

### Windows
[WIP]

## Usage
[WIP]

## Building
This project is developed in 🔥blazingly-fast, memory-safe🔥 Rust. Building and running it from source should be relatively simple using the Rust toolchain/Cargo. You can find installation instructions here [rustup.rs](https://rustup.rs/).

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
## Licensing & Attributions
This project is licensed under the **GNU General Public License v3.0**.

Upon first use of the ``Kihon`` plugin, the following datasets are downloaded and are the property of their respective owners:
 - **JMdict-Simplified:** A JSON conversion of the JMdict dictionary files provided by [**scriptin/jmdict-simplified**](https://github.com/scriptin/jmdict-simplified) (specifically [jmdict-eng-3.6.2+20260202123847](https://github.com/scriptin/jmdict-simplified/releases/download/3.6.2%2B20260202123847/jmdict-eng-3.6.2+20260202123847.json.tgz)) under the **CC BY-SA 4.0 License**.
 - **JmdictFurigana:** Furigana data for the JMdict provided by [**Doublevil/JmdictFurigana**](https://github.com/Doublevil/JmdictFurigana) (specifically [2.3.1+2026-01-25](https://github.com/Doublevil/JmdictFurigana/releases/download/2.3.1%2B2026-01-25/JmdictFurigana.json)) under the **MIT License**.
   - **JMdict:** The original JMdict XML files are property of the **Electronic Dictionary Research and Development Group**, used in accordance with the [**EDRDG Licence**](https://www.edrdg.org/edrdg/licence.html).

 - **Word Frequencies:** A list of Japanese words by frequency provided by [**hingston/japanese**](https://github.com/hingston/japanese) (specifically [44492-japanese-words-latin-lines-removed](https://github.com/hingston/japanese/blob/78a5f64e872e4a2ad430adfd124c98f5f0a1619b/44492-japanese-words-latin-lines-removed.txt)).
   - **University of Leeds Corpus:** The word frequencies are based on the [**University of Leeds Corpus**](https://web.archive.org/web/20230924010025/http://corpus.leeds.ac.uk/frqc/internet-jp.num), used in accordance with the **CC BY-SA 2.5 License**.

When using the ``Kihon`` plugin, the following is used:
 - **Vibrato:** The Vibrato tokenizer provided by [**daac-tools/vibrato**](https://github.com/daac-tools/vibrato) under the **MIT License**. For tokenization the [jumandic-mecab-7_0](https://github.com/daac-tools/vibrato/releases/download/v0.5.0/jumandic-mecab-7_0.tar.xz) file is downloaded (if not already present).

   - **JumanDIC:** The JumanDIC is the property of **Kyoto University** and is provided at [ku-nlp/JumanDIC](https://github.com/ku-nlp/JumanDIC).
