# ffpb-rs

**Not smart. Not comprehensive. Not guaranteed to work.**

<p align="center">
  <a href="https://crates.io/crates/ffpb">
    <img src="https://img.shields.io/crates/d/ffpb?style=flat-square">
  </a>
  <a href="https://crates.io/crates/ffpb">
    <img src="https://img.shields.io/crates/v/ffpb?style=flat-square">
  </a>
  <a href="https://docs.rs/ffpb/latest/ffpb">
    <img src="https://img.shields.io/docsrs/ffpb?logo=docsdotrs&style=flat-square">
  </a>
  <a href="https://github.com/clitic/ffpb#license">
    <img src="https://img.shields.io/crates/l/ffpb?style=flat-square">
  </a>
  <a href="https://github.com/clitic/ffpb-rs">
    <img src="https://img.shields.io/github/repo-size/clitic/ffpb-rs?style=flat-square">
  </a>
</p>

ffpb-rs is rust implementation of [ffpb](https://github.com/althonos/ffpb).
ffpb is an ffmpeg progress formatter. It will attempt to display a nice progress bar in the output, based on the raw ffmpeg output, as well as an adaptative ETA timer.

![showcase](https://raw.githubusercontent.com/clitic/ffpb-rs/main/images/showcase.gif)

## Installations

Visit [releases](https://github.com/clitic/ffpb-rs/releases) for prebuilt binaries. You just need to copy that binary to any path specified in your `PATH` environment variable.

Or you can even install it through cargo.

```bash
cargo install ffpb
```

## Usage

ffpb is is not even self-aware. Any argument given to the ffpb command is transparently given to the ffmpeg binary on your system, without any form of validation. So if you know how to use the ffmpeg cli, you know how to use ffpb.

```bash
ffpb --help
```

## Rust Library

Add this to your Cargo.toml file.

```toml
[dependencies]
ffpb = "0.1.2"
```

 Then call ffmpeg like this.

```rust
fn main() {
    let args = ["-i", "test.mp4", "-c:v", "copy", "test.mkv"]
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>();

    ffpb::ffmpeg(&args).unwrap();
}
```

## License

&copy; 2022-24 clitic

This repository is licensed under the MIT license. See LICENSE for details.
