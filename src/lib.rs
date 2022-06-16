use std::io::{BufRead, Read, Write};

fn new_error(msg: &str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, msg)
}

fn time_to_secs(x: &regex::Captures) -> Result<usize, std::num::ParseIntError> {
    let hours = x.get(1).unwrap().as_str().parse::<usize>()?;
    let minutes = x.get(2).unwrap().as_str().parse::<usize>()?;
    let seconds = x.get(3).unwrap().as_str().parse::<usize>()?;
    Ok((((hours * 60) + minutes) * 60) + seconds)
}

/// Call ffmpeg command with args and coloured progress bar.
///
/// # Example
///
/// ```rust
/// fn main() {
///     let args = ["-i", "test.mp4", "-c:v", "copy", "test.mkv"]
///     .iter()
///     .map(|x| x.to_string())
///     .collect::<Vec<String>>();
///
///     ffpb::ffmpeg(&args).unwrap();
/// }
/// ```
pub fn ffmpeg(args: &[String]) -> Result<(), std::io::Error> {
    let ffmpeg = std::process::Command::new("ffmpeg")
        .args(args)
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|_| new_error("Failed to launch ffmpeg binary."))?
        .stderr
        .ok_or_else(|| new_error("Failed to capture ffmpeg standard error."))?;

    let mut reader = std::io::BufReader::new(ffmpeg);
    let mut pb = kdam::RichProgress::new(
        kdam::tqdm!(unit = " second".to_owned(), dynamic_ncols = true),
        vec![
            kdam::Column::Bar,
            kdam::Column::Percentage(2),
            kdam::Column::Text("•".to_owned(), None),
            kdam::Column::CountTotal,
            kdam::Column::Text("•".to_owned(), None),
            kdam::Column::Text("".to_owned(), None),
            kdam::Column::Text("•".to_owned(), None),
            kdam::Column::ElapsedTime,
            kdam::Column::Text("ET".to_owned(), Some("cyan".to_owned())),
            kdam::Column::Text("•".to_owned(), None),
            kdam::Column::RemainingTime,
            kdam::Column::Text("ETA".to_owned(), Some("cyan".to_owned())),
        ],
    );

    let mut duration = None;
    let mut fps = None;
    let mut check_overwrite = true;
    let mut read_byte = b'\n';

    let duration_rx = regex::Regex::new(r"Duration: (\d{2}):(\d{2}):(\d{2})\.\d{2}").unwrap();
    let fps_rx = regex::Regex::new(r"(\d{2}\.\d{2}|\d{2}) fps").unwrap();
    let progress_rx = regex::Regex::new(r"time=(\d{2}):(\d{2}):(\d{2})\.\d{2}").unwrap();

    loop {
        let mut prepend_text = String::from("");

        if check_overwrite {
            let mut pre_buf = [0; 5];
            reader
                .read_exact(&mut pre_buf)
                .map_err(|_| new_error("No such file or directory."))?;
            prepend_text.push_str(&String::from_utf8_lossy(&pre_buf));

            match prepend_text.as_str() {
                "File " => {
                    let mut post_buf = vec![];
                    reader.read_until(b']', &mut post_buf)?;

                    eprint!("File {} ", String::from_utf8(post_buf).unwrap());

                    std::io::stderr().flush().unwrap();
                    check_overwrite = false;
                    read_byte = b'\r';
                }

                "frame" => {
                    check_overwrite = false;
                    read_byte = b'\r';
                }

                _ => (),
            }
        } else {
            std::thread::sleep(std::time::Duration::from_secs_f32(0.1));
        }

        let mut buf = vec![];
        reader.read_until(read_byte, &mut buf)?;

        if let Ok(line) = String::from_utf8(buf) {
            let std_line = prepend_text + &line;

            if std_line == "" {
                pb.refresh();
                eprintln!();
                break;
            }

            if duration.is_none() {
                if let Some(x) = duration_rx.captures_iter(&std_line).next() {
                    duration = Some(
                        time_to_secs(&x)
                            .map_err(|_| new_error("Couldn't parse total duration."))?,
                    );
                    pb.pb.total = duration.unwrap();
                }
            }

            if fps.is_none() {
                if let Some(x) = fps_rx.captures_iter(&std_line).next() {
                    fps = Some(
                        x.get(1)
                            .unwrap()
                            .as_str()
                            .parse::<f32>()
                            .map_err(|_| new_error("Couldn't parse fps."))?,
                    );
                    pb.pb.unit = " frame".to_owned();
                }
            }

            if let Some(x) = progress_rx.captures_iter(&std_line).next() {
                let mut current =
                    time_to_secs(&x).map_err(|_| new_error("Couldn't parse current duration."))?;

                if let Some(frames) = fps {
                    current *= frames as usize;
                    if pb.pb.total == duration.unwrap_or(0) {
                        pb.pb.total *= frames as usize;
                    }
                }

                pb.replace(
                    5,
                    kdam::Column::Text(format!("{:.0} FPS", pb.pb.rate()), Some("red".to_owned())),
                );

                if current >= pb.pb.total {
                    pb.pb.clear();
                    eprintln!("\r{}", std_line.replace("\r", "").trim_end_matches("\n"));
                }

                pb.set_position(current);
            }
        } else {
            break;
        }
    }

    Ok(())
}
