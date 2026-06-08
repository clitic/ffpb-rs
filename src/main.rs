fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();

    if args.iter().any(|a| a == "-h" || a == "--help") {
        eprintln!("ffmpeg with a progress bar.\n",);
        eprintln!("\x1b[1mUsage:\x1b[0m ffpb [ffmpeg arguments...]\n");
        eprintln!("\x1b[1mOptions:\x1b[0m");
        eprintln!("  --clean          Only show progress bar, suppress ffmpeg output");
        eprintln!("  -h, --help       Show this help");
        eprintln!("  -V, --version    Show ffpb version\n");
        eprintln!("\x1b[1mExamples:\x1b[0m");
        eprintln!("  ffpb -i input.mp4 -c:v libx264 output.mp4");
        eprintln!("  ffpb -ss 10 -to 20 -i input.mp4 output.mp4");
        eprintln!("  ffpb -y -i input.mp4 -c:a aac output.m4a\n");
        eprintln!("All other arguments are forwarded directly to ffmpeg.");
        return;
    }
    if args.iter().any(|a| a == "-V" || a == "--version") {
        println!("ffpb {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    let code = ffpb::run(&args).unwrap_or_else(|e| {
        eprintln!("\x1b[1;31m[ERROR]\x1b[0m {e}");
        1
    });
    std::process::exit(code);
}
