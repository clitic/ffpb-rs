use kdam::term::Colorizer;
use std::io::{BufRead, Read, Write};

const USAGE: &str = "ffpb v0.1.3
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

fn new_error(msg: &str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, msg)
}

fn time_to_secs(x: &regex::Captures) -> Result<usize, std::num::ParseIntError> {
    let hours = x.get(1).unwrap().as_str().parse::<usize>()?;
    let minutes = x.get(2).unwrap().as_str().parse::<usize>()?;
    let seconds = x.get(3).unwrap().as_str().parse::<usize>()?;
    Ok((((hours * 60) + minutes) * 60) + seconds)
}

fn ffmpeg(args: &[String]) -> Result<(), std::io::Error> {
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
    // let mut pb = kdam::tqdm!(unit = " second".to_owned(), dynamic_ncols = true);

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

    loop {
        let mut prepend_text = String::from("");

        if check_overwrite {
            let mut pre_buf = [0; 5];
            reader
                .read_exact(&mut pre_buf)
                .map_err(|_| new_error("No such file or directory."))?;
            prepend_text.push_str(&String::from_utf8_lossy(&pre_buf));

            if prepend_text.contains("File") {
                let mut post_buf = vec![];
                reader.read_until(b']', &mut post_buf)?;
                eprint!("File{} ", String::from_utf8(post_buf).unwrap());
                std::io::stderr().flush().unwrap();
                check_overwrite = false;
                read_byte = b'\r';
            } else if prepend_text.starts_with("\nframe=") || prepend_text.starts_with("frame=") {
                check_overwrite = false;
                read_byte = b'\r';
            }

            if pb.pb.n != 0 {
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
                pb.set_position(current);
            }
        } else {
            break;
        }
    }

    Ok(())
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    if args[1..].len() == 0 || args[1] == "-h" || args[1] == "--help" {
        print!("{}", USAGE);
    }

    if let Err(e) = ffmpeg(&args[1..]) {
        eprintln!("{}", e.to_string().colorize("bold red"));
        std::process::exit(1);
    }
}
