use std::{
    error::Error,
    io::Write,
    process::{Child, Command, Stdio},
};

pub fn check_tesseract() -> Result<(), Box<dyn Error>> {
    Command::new("tesseract").arg("--version").output()?;
    Ok(())
}

pub fn ocr_image(image_data: &[u8]) -> Result<String, Box<dyn Error>> {
    check_tesseract()?;

    let ver_conf_command = Command::new("tesseract")
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
            let hor_conf_command = Command::new("tesseract")
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
                let hor_command = Command::new("tesseract")
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
                let ver_command = Command::new("tesseract")
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
            let hor_command = Command::new("tesseract")
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
