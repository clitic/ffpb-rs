use kdam::term::Colorizer;
use std::{env, process};

const USAGE: &str = "ffpb 0.1.2
clitic <clitic21@gmail.com>

A coloured progress bar for ffmpeg. ffpb-rs is rust implementation of https://github.com/althonos/ffpb.

ffpb is an ffmpeg progress formatter. It will attempt to display a nice progress bar in the output, based on the raw ffmpeg output, as well as an adaptative ETA timer.

ffpb is is not even self-aware. Any argument given to the ffpb command is transparently given to the ffmpeg binary on your system, without any form of validation. So if you know how to use the ffmpeg cli, you know how to use ffpb.

USAGE:
  ffpb <FFMPEG OPTIONS>

EXAMPLES:
  ffpb -i test.mkv test.mp4
  ffpb -i test.mkv -c:v copy test.mp4
";

fn main() {
    let vs = process::Command::new("ffmpeg")
        .args([
            "-loglevel",
            "fatal",
            "-i",
            r"D:\test.mp4",
            "-f",
            "yuv4mpegpipe",
            "-strict",
            "-1",
            "-",
        ])
        .stdout(process::Stdio::piped())
        .spawn()
        .expect("Failed in spawning FFmpeg child");

    let args: Vec<String> = [
        "-f",
        "yuv4mpegpipe",
        "-i",
        "-",
        "-c:v",
        "libx264",
        "-preset",
        "medium",
        "-crf",
        "23",
        "output_video.mp4",
        "-y",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();

    let _a = ffpb::ffmpeg(
        "ffmpeg".to_owned(),
        &args,
        vs.stdout.expect("Failed to open vspipe stdout"),
    );

    dbg!(&_a);
}
