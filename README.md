<h1 align="center">ffpb-rs</h1>

[![Github Downloads](https://img.shields.io/github/downloads/clitic/ffpb-rs/total?logo=github&style=flat-square)](https://github.com/clitic/ffpb-rs/releases)
[![Crate Downloads](https://img.shields.io/crates/d/ffpb?logo=rust&style=flat-square)](https://crates.io/ffpb)
[![Crate Version](https://img.shields.io/crates/v/ffpb?style=flat-square)](https://crates.io/ffpb)
[![Build Status](https://img.shields.io/github/actions/workflow/status/clitic/ffpb-rs/build.yml?logo=github&style=flat-square)](https://github.com/clitic/ffpb-rs/actions)
[![Docs Status](https://img.shields.io/docsrs/ffpb?logo=docsdotrs&style=flat-square)](https://docs.rs/ffpb)
[![Crate License](https://img.shields.io/crates/l/ffpb?style=flat-square)](https://crates.io/ffpb)
[![Repo Size](https://img.shields.io/github/repo-size/clitic/ffpb-rs?logo=github&style=flat-square)](https://github.com/clitic/ffpb-rs)

`ffpb` is a modern, cli progress bar for ffmpeg. It was originally inspired from [althonos/ffpb](https://github.com/althonos/ffpb). It seamlessly wraps your `ffmpeg` commands, parses the `-progress` output, and replaces ffmpeg's standard console spam with a clean, dynamic, and beautiful progress bar featuring an adaptive ETA and real-time encoding statistics.

![showcase](https://raw.githubusercontent.com/clitic/ffpb-rs/main/images/showcase.gif)

## Installation
  
### Dependencies

- [ffmpeg](https://www.ffmpeg.org/download.html) a free, open-source command-line software framework used for handling multimedia files.

### Pre-built Binaries

Visit the [releases page](https://github.com/clitic/ffpb-rs/releases) for pre-built binaries. Extract the binary and add its path to your system's `PATH`.

### Install via Cargo

You can also install ffpb using cargo.

```bash
cargo install ffpb
```

## Usage

```
ffmpeg with a progress bar.

Usage: ffpb [ffmpeg arguments...]

Options:
  --clean          Only show progress bar, suppress ffmpeg output
  -h, --help       Show this help
  -V, --version    Show ffpb version

Examples:
  ffpb -i input.mp4 -c:v libx264 output.mp4
  ffpb -ss 10 -to 20 -i input.mp4 output.mp4
  ffpb -y -i input.mp4 -c:a aac output.m4a

All other arguments are forwarded directly to ffmpeg.
```

## Library

Add this to your Cargo.toml file.

```toml
[dependencies]
ffpb = "0.2.0"
```

Or add from command line.

```bash
cargo add ffpb
```

See [docs](https://docs.rs/ffpb) and [examples](https://github.com/clitic/ffpb-rs/blob/main/examples) to 
know how to use it.

## License

Dual Licensed

- [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0) ([LICENSE-APACHE](LICENSE-APACHE))
- [MIT license](https://opensource.org/licenses/MIT) ([LICENSE-MIT](LICENSE-MIT))
