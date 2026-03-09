use std::{
    error::Error,
    io::Write,
    process::{Child, Command, Stdio},
};

const TESS_PATH_WINDOWS: &str = "C:\\Program Files\\Tesseract-OCR\\tesseract.exe";

pub fn check_tesseract() -> Result<String, Box<dyn Error>> {
    if let Err(e) = Command::new("tesseract").arg("--version").output() {
        #[cfg(target_os = "linux")]
        return Err(Box::new(e));
        // tesseract installer on Windows doesn't set PATH automatically, check default dir without PATH
        #[cfg(target_os = "windows")]
        {
            Command::new(TESS_PATH_WINDOWS).arg("--version").output()?;
            return Ok(String::from(
                "C:\\Program Files\\Tesseract-OCR\\tesseract.exe",
            ));
        }
    }
    return Ok(String::from("tesseract"));
}

pub fn ocr_image(image_data: &[u8]) -> Result<String, Box<dyn Error>> {
    let tess_command: String = check_tesseract()?;

    let ver_conf_command = Command::new(&tess_command)
        .arg("stdin")
        .arg("stdout")
        .arg("-l")
        .arg("jpn_vert")
        .arg("--psm")
        .arg("5")
        .arg("tsv")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    match get_conf(ver_conf_command, image_data) {
        Ok(ver_conf) => {
            let hor_conf_command = Command::new(&tess_command)
                .arg("stdin")
                .arg("stdout")
                .arg("-l")
                .arg("jpn")
                .arg("tsv")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;
            let hor_conf: f32 = get_conf(hor_conf_command, image_data)?;
            if hor_conf >= ver_conf {
                let hor_command = Command::new(&tess_command)
                    .arg("stdin")
                    .arg("stdout")
                    .arg("-l")
                    .arg("jpn")
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()?;
                let text = get_text(hor_command, image_data)?;
                Ok(text)
            } else {
                let ver_command = Command::new(&tess_command)
                    .arg("stdin")
                    .arg("stdout")
                    .arg("-l")
                    .arg("jpn_vert")
                    .arg("--psm")
                    .arg("5")
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()?;
                let text = get_text(ver_command, image_data)?;
                Ok(text)
            }
        }
        Err(_) => {
            eprintln!(
                "Couldn't parse vertical text. Make sure you have jpn_vert.traineddata installed if you want vertical text support."
            );
            let hor_command = Command::new(&tess_command)
                .arg("stdin")
                .arg("stdout")
                .arg("-l")
                .arg("jpn")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;
            let text = get_text(hor_command, image_data)?;
            Ok(text)
        }
    }
}

fn get_conf(mut child: Child, image_data: &[u8]) -> Result<f32, Box<dyn Error>> {
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(image_data)?;
    }
    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(Box::from("Error when trying to call tesseract."));
    }
    let tsv = String::from_utf8_lossy(&output.stdout);

    let mut total_conf: f32 = 0.0;
    let mut count: i32 = 0;

    for line in tsv.lines().skip(1) {
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() >= 11 {
            if let Ok(conf) = fields[10].parse::<f32>() {
                if conf >= 0.0 {
                    total_conf += conf;
                    count += 1;
                }
            }
        }
    }

    Ok(total_conf / count as f32)
}

fn get_text(mut child: Child, image_data: &[u8]) -> Result<String, Box<dyn Error>> {
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(image_data)?;
    }
    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(Box::from("Error when trying to call tesseract."));
    }
    let text = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(text)
}
