//! A library and cli wrapper around ffmpeg that displays a modern, colored
//! progress bar with encoding statistics.

mod args;
mod progress;
mod runner;

/// Run ffmpeg with a built-in progress bar.
///
/// Takes the same arguments you'd pass to `ffmpeg` on the command line.
/// Returns the ffmpeg exit code.
///
/// # Example
/// ```no_run
/// let args = vec!["-y", "-i", "input.mp4", "output.mp4"]
///     .into_iter().map(String::from).collect::<Vec<_>>();
/// let code = ffpb::run(&args).unwrap();
/// std::process::exit(code);
/// ```
pub fn run(args: &[String]) -> Result<i32, Error> {
    runner::run_ffmpeg(&args::parse_args(args))
}

/// Error type for ffpb operations.
#[derive(Debug)]
pub enum Error {
    /// Failed to spawn the ffmpeg process.
    SpawnFailed(std::io::Error),
    /// FFmpeg was not found in PATH.
    FfmpegNotFound,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::SpawnFailed(e) => write!(f, "failed to spawn ffmpeg ({e})"),
            Error::FfmpegNotFound => write!(f, "ffmpeg not found in PATH"),
        }
    }
}

impl std::error::Error for Error {}
