use std::error::Error;
use std::fs::File;
use std::io;
use std::path::PathBuf;
use tar::Archive;
use xz2::read::XzDecoder;

const JUMANDIC_URL: &str =
    "https://github.com/daac-tools/vibrato/releases/download/v0.5.0/jumandic-mecab-7_0.tar.xz";

pub fn fetch_jumandic(destination_path: &PathBuf) -> Result<(), Box<dyn Error>> {
    let response = reqwest::blocking::get(JUMANDIC_URL)?;

    let xz_decoder = XzDecoder::new(response);
    let mut archive = Archive::new(xz_decoder);

    for entry_result in archive.entries()? {
        let entry = entry_result?;
        let path = entry.path()?;

        if path.ends_with("system.dic.zst") {
            let mut zstd_decoder = zstd::stream::read::Decoder::new(entry)?;

            if let Some(parent) = destination_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut out_file = File::create(destination_path)?;

            io::copy(&mut zstd_decoder, &mut out_file)?;

            return Ok(());
        }
    }

    return Err(Box::from("No system dictionary found in archive"));
}
