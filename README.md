<h1 align="center">ffpb-rs</h1>

[![Github Downloads](https://img.shields.io/github/downloads/clitic/ffpb-rs/total?logo=github&style=flat-square)](https://github.com/clitic/ffpb-rs/releases)
[![Crate Downloads](https://img.shields.io/crates/d/ffpb?logo=rust&style=flat-square)](https://crates.io/ffpb)
[![Crate Version](https://img.shields.io/crates/v/ffpb?style=flat-square)](https://crates.io/crates/ffpb)
[![Build Status](https://img.shields.io/github/actions/workflow/status/clitic/ffpb-rs/build.yml?logo=github&style=flat-square)](https://github.com/clitic/ffpb-rs/actions)
[![Docs Status](https://img.shields.io/docsrs/ffpb?logo=docsdotrs&style=flat-square)](https://docs.rs/ffpb)
[![Crate License](https://img.shields.io/crates/l/ffpb?style=flat-square)](https://crates.io/crates/ffpb)
[![Repo Size](https://img.shields.io/github/repo-size/clitic/ffpb-rs?logo=github&style=flat-square)](https://github.com/clitic/ffpb-rs)

`ffpb` is a modern, cli progress bar for ffmpeg. It was originally inspired from [althonos/ffpb](https://github.com/althonos/ffpb). It seamlessly wraps your `ffmpeg` commands, parses the `-progress` output, and replaces ffmpeg's standard console spam with a clean, dynamic, and beautiful progress bar featuring an adaptive ETA and real-time encoding statistics.

<div align="center">
  <img src="https://raw.githubusercontent.com/clitic/ffpb-rs/refs/heads/main/images/showcase.gif" width="700px">
</div>

## Features

- **Drop-in Replacement**: Simply replace `ffmpeg` with `ffpb` in your existing `ffmpeg` commands. No complex configurations needed.
- **Beautiful UI**: Modern, true-color gradient progress bar that adapts to your terminal size.
- **Real-Time Stats**: Displays frames, fps, q-value, size, elapsed time, ETA, bitrate, and speed.
- **Smart Duration Parsing**: Automatically parses `-t`, `-to`, and `-ss` flags to accurately compute the effective encoding duration.

## Installation
  
### Dependencies

- [ffmpeg](https://www.ffmpeg.org/download.html) is a free, open-source command-line software framework used for handling multimedia files.

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
ffpb = "0.2.1"
```

Or add from command line.

```bash
cargo add ffpb
```

See [docs](https://docs.rs/ffpb) and [examples](https://github.com/clitic/ffpb-rs/blob/main/examples) to 
know how to use it.

## Donate

This project is developed and maintained in my free time. Donations help cover development time, testing, and future improvements. If this tool saved you time or helped your workflow, consider supporting it.

<div align="center">
  <a href="mailto:clitic21@gmail.com" target="_blank" style="text-decoration: none;">
    <img src="https://raw.githubusercontent.com/clitic/vsd/refs/heads/main/docs/assets/contact.svg" alt="Contact Me" height="42px">
  </a>
  <a href="https://ko-fi.com/clitic" target="_blank" style="text-decoration: none;">
    <img src="https://storage.ko-fi.com/cdn/kofi5.png?v=6" alt="Buy Me a Coffee at ko-fi.com" height="40px" />
  </a>
  <a href="https://www.buymeacoffee.com/clitic" target="_blank" style="text-decoration: none;">
    <img src="https://raw.githubusercontent.com/clitic/vsd/refs/heads/main/docs/assets/bmc.svg" alt="Buy Me A Coffee" height="40px">
  </a>
  <a href="https://paypal.me/clitic" target="_blank" style="text-decoration: none;">
    <img src="https://raw.githubusercontent.com/clitic/vsd/refs/heads/main/docs/assets/paypal.svg" alt="PayPal" height="40px">
  </a>
</div>

## License

Dual Licensed

- [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0) ([LICENSE-APACHE](LICENSE-APACHE))
- [MIT license](https://opensource.org/licenses/MIT) ([LICENSE-MIT](LICENSE-MIT))
