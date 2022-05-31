use kdam::term::Colorizer;

const USAGE: &str = "ffpb v0.1.0
clitic <clitic21@gmail.com>
A coloured progress bar for ffmpeg. ffpb-rs is rust implementation of https://github.com/althonos/ffpb.

ffpb is an ffmpeg progress formatter. It will attempt to display a nice progress bar in the output, based on the raw ffmpeg output, as well as an adaptative ETA timer.

ffpb is is not even self-aware. Any argument given to the ffpb command is transparently given to the ffmpeg binary on your system, without any form of validation. So if you know how to use the ffmpeg cli, you know how to use ffpb.

USAGE:
    ffpb [ffmpeg <OPTIONS>]

EXAMPLES:
    ffpb -i test.mkv test.mp4
    ffpb -i test.mkv -c:v copy test.mp4
";

fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    if args[1..].len() == 0 || args[1] == "-h" || args[1] == "--help" {
        print!("{}", USAGE);
    } else {
        if let Err(e) = ffpb::ffmpeg(&args[1..]) {
            eprintln!("{}", e.to_string().colorize("bold red"));
            std::process::exit(1);
        }
    }
}
