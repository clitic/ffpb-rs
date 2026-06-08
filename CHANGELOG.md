# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-06-08

### Added

- New `--clean` flag to suppress all `ffmpeg` output and only show the progress bar.

### Changed

- Complete rewrite of the progress bar rendering engine.
- Modernized UI with a true-color gradient, micro-animations for indeterminate states, and dot-separated stats.

## [0.1.2] - 2024-05-01

### Added

- Bump up kdam to `v0.5`.
- Bump up regex to `v1.10`.

### Fixed

- When output file name contains `]` character. A complete message for overwrite is displayed.

## [0.1.1] - 2022-06-16

### Fixed

- Extra removals of `\n` characters from ffmpeg output.

## [0.1.0] - 2022-05-31

[Unreleased]: https://github.com/clitic/ffpb-rs/compare/0.2.0...HEAD
[0.2.0]: https://github.com/clitic/ffpb-rs/compare/0.1.2...0.2.0
[0.1.2]: https://github.com/clitic/ffpb-rs/compare/0.1.1...0.1.2
[0.1.1]: https://github.com/clitic/ffpb-rs/compare/0.1.0...0.1.1
[0.1.0]: https://github.com/clitic/ffpb-rs/compare/27d4808...0.1.0
