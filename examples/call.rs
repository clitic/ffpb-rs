fn main() {
    let args = ["-i", "test.mp4", "-c:v", "copy", "test.mkv"]
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>();

    ffpb::ffmpeg(&args).unwrap();
}
