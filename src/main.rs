use std::{
    env,
    io::{self, Write},
    path::PathBuf,
    process::{self, Command},
};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.len() > 0 {
        if args.len() > 1 {
            let query = if args[0] == "-ocr" || args[0] == "-ocrv" {
                args[1..].join("")
            } else {
                args.join("")
            };
            if let Err(e) = popup_dictionary::run(&query) {
                eprintln!("Error: {e}");
                process::exit(1);
            }
        } else {
            if args[0] == "-ocr" {
                let linux_copy_path: PathBuf = match dirs::config_dir() {
                    Some(path) => path.join("popup_dictionary/linux_ocr.sh"),
                    None => {
                        eprintln!("No valid config path found in environment variables.");
                        process::exit(1);
                    }
                };
                let output: process::Output = Command::new("sh")
                    .arg(linux_copy_path)
                    .output()
                    .expect("failed to execute linux copy");
                println!("status: {}", output.status);
                io::stdout().write_all(&output.stdout).unwrap();
                io::stderr().write_all(&output.stderr).unwrap();
            } else if args[0] == "-ocrv" {
                let linux_copy_path: PathBuf = match dirs::config_dir() {
                    Some(path) => path.join("popup_dictionary/linux_ocrv.sh"),
                    None => {
                        eprintln!("No valid config path found in environment variables.");
                        process::exit(1);
                    }
                };
                let output: process::Output = Command::new("sh")
                    .arg(linux_copy_path)
                    .output()
                    .expect("failed to execute linux copy");
                println!("status: {}", output.status);
                io::stdout().write_all(&output.stdout).unwrap();
                io::stderr().write_all(&output.stderr).unwrap();
            } else {
                let query: String = args.join("");
                if let Err(e) = popup_dictionary::run(&query) {
                    eprintln!("Error: {e}");
                    process::exit(1);
                }
            }
        }
    } else {
        if cfg!(target_os = "windows") {
            //Command::new("cmd")
        } else {
            let linux_copy_path: PathBuf = match dirs::config_dir() {
                Some(path) => path.join("popup_dictionary/linux_copy.sh"),
                None => {
                    eprintln!("No valid config path found in environment variables.");
                    process::exit(1);
                }
            };
            let output: process::Output = Command::new("sh")
                .arg(linux_copy_path)
                .output()
                .expect("failed to execute linux copy");
            println!("status: {}", output.status);
            io::stdout().write_all(&output.stdout).unwrap();
            io::stderr().write_all(&output.stderr).unwrap();
        }
    }
}
