use std::io::{BufRead, BufReader, Error, ErrorKind, Read, Write};

const USAGE: &str = "ffpb v0.1.1
clitic <clitic21@gmail.com>
A progress bar for ffmpeg. ffpb-rs is rust implementation of https://github.com/althonos/ffpb.

ffpb is an ffmpeg progress formatter. It will attempt to display a nice progress bar in the output, based on the raw ffmpeg output, as well as an adaptative ETA timer.

ffpb is is not even self-aware. Any argument given to the ffpb command is transparently given to the ffmpeg binary on your system, without any form of validation. So if you know how to use the ffmpeg cli, you know how to use ffpb.

USAGE:
    ffpb [ffmpeg <OPTIONS>]

EXAMPLES:
    ffpb -i test.mkv test.mp4
    ffpb -i test.mkv -c:v copy test.mp4
";

fn time_to_secs(x: &regex::Captures) -> Result<usize, std::num::ParseIntError> {
    let hours = x.get(1).unwrap().as_str().parse::<usize>()?;
    let minutes = x.get(2).unwrap().as_str().parse::<usize>()?;
    let seconds = x.get(3).unwrap().as_str().parse::<usize>()?;
    Ok((((hours * 60) + minutes) * 60) + seconds)
}

fn main() -> Result<(), Error> {
    let args = std::env::args().collect::<Vec<String>>();

    if args[1..].len() == 0 || args[1] == "-h" || args[1] == "--help" {
        print!("{}", USAGE);
        return Ok(());
    }

    let ffmpeg = std::process::Command::new("ffmpeg")
        .args(&args[1..])
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to launch ffmpeg binary.");

    let ffmpeg_stderr = ffmpeg
        .stderr
        .ok_or_else(|| Error::new(ErrorKind::Other, "Failed to capture ffmpeg standard error."))?;
    // .expect("Failed to capture ffmpeg standard error.")?;

    let mut reader = BufReader::new(ffmpeg_stderr);
    let mut pb = kdam::tqdm!(
        unit_scale = true,
        unit = "".to_owned(),
        dynamic_ncols = true
    );
    let mut duration = None;
    let mut fps = None;
    let mut check_overwrite = true;
    let mut read_byte = if cfg!(target_os = "windows") {
        b'\r'
    } else {
        b'\n'
    };

    let duration_rx = regex::Regex::new(r"Duration: (\d{2}):(\d{2}):(\d{2})\.\d{2}").unwrap();
    let fps_rx = regex::Regex::new(r"(\d{2}\.\d{2}|\d{2}) fps").unwrap();
    let progress_rx = regex::Regex::new(r"time=(\d{2}):(\d{2}):(\d{2})\.\d{2}").unwrap();
    let size_rx = regex::Regex::new(r"(\d*)kB time").unwrap();
    let bitrate_rx = regex::Regex::new(r"(\d*\.\d*)kbits/s speed").unwrap();
    let speed_rx = regex::Regex::new(r"speed=(\s|)(.*x)").unwrap();

    loop {
        let mut prepend_text = String::from("");

        if check_overwrite {
            let mut pre_buf = [0; 6];
            reader
                .read_exact(&mut pre_buf)
                .expect("No such file or directory.");
            prepend_text.push_str(&String::from_utf8_lossy(&pre_buf));

            if prepend_text.contains("File ") {
                let mut post_buf = vec![];
                reader.read_until(b']', &mut post_buf)?;
                print!("File {} ", String::from_utf8(post_buf).unwrap());
                std::io::stdout().flush()?;
                check_overwrite = false;
                read_byte = b'\r';
            } else if prepend_text.starts_with("\nframe=") || prepend_text.starts_with("frame=") {
                check_overwrite = false;
                read_byte = b'\r';
            }

            if pb.n != 0 {
                check_overwrite = false;
                read_byte = b'\r';
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
                println!();
                break;
            }

            if duration.is_none() {
                if let Some(x) = duration_rx.captures_iter(&std_line).next() {
                    duration = Some(time_to_secs(&x).expect("Couldn't parse total duration."));
                    pb.total = duration.unwrap();
                }
            }

            if fps.is_none() {
                if let Some(x) = fps_rx.captures_iter(&std_line).next() {
                    fps = Some(
                        x.get(1)
                            .unwrap()
                            .as_str()
                            .parse::<f32>()
                            .expect("Couldn't parse fps."),
                    );
                }
            }

            if let Some(x) = progress_rx.captures_iter(&std_line).next() {
                let mut current = time_to_secs(&x).expect("Couldn't parse current duration.");

                if let Some(frames) = fps {
                    current *= frames as usize;
                    if pb.total == duration.unwrap_or(0) {
                        pb.total *= frames as usize;
                    }
                }

                let mut postfix = String::new();

                if let Some(size) = size_rx.captures_iter(&std_line).next() {
                    let size_in_bytes = size
                        .get(1)
                        .unwrap()
                        .as_str()
                        .parse::<usize>()
                        .expect("Couldn't parse size.")
                        * 1024;
                    postfix += &kdam::format::format_sizeof(size_in_bytes, 1024);
                    postfix += "B";
                }
                
                if let Some(bitrate) = bitrate_rx.captures_iter(&std_line).next() {
                    postfix += ", ";
                    postfix += bitrate.get(1).unwrap().as_str();
                    postfix += "kB/s";
                }

                if let Some(speed) = speed_rx.captures_iter(&std_line).next() {
                    postfix += ", ";
                    postfix += speed.get(2).unwrap().as_str();
                }

                pb.set_postfix(postfix);
                pb.set_position(current);
            }
        } else {
            break;
        }
    }

    Ok(())
}
