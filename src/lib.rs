use kdam::{tqdm, BarExt, Column, RichProgress};
use regex::{Captures, Regex};
use std::{
    io::{stderr, BufRead, BufReader, Error, ErrorKind, IsTerminal, Read, Write},
    num::ParseIntError,
    process::{Command, Stdio},
    thread,
    time::Duration,
};

fn new_error(msg: &str) -> Error {
    Error::new(ErrorKind::Other, msg)
}

fn time_to_secs(x: &Captures) -> Result<usize, ParseIntError> {
    let hours = x.get(1).unwrap().as_str().parse::<usize>()?;
    let minutes = x.get(2).unwrap().as_str().parse::<usize>()?;
    let seconds = x.get(3).unwrap().as_str().parse::<usize>()?;
    Ok((((hours * 60) + minutes) * 60) + seconds)
}

/// Call ffmpeg command with args and coloured progress bar.
///
/// # Example
///
/// ```rust
/// fn main() {
///     let args = ["-i", "test.mp4", "-c:v", "copy", "test.mkv"]
///     .iter()
///     .map(|x| x.to_string())
///     .collect::<Vec<String>>();
///
///     ffpb::ffmpeg(&args).unwrap();
/// }
/// ```
pub fn ffmpeg(args: &[String]) -> Result<(), Error> {
    kdam::term::init(stderr().is_terminal());

    let ffmpeg = Command::new("ffmpeg")
        .args(args)
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| new_error("failed to launch ffmpeg binary."))?
        .stderr
        .ok_or_else(|| new_error("failed to capture ffmpeg standard error."))?;

    let mut reader = BufReader::new(ffmpeg);
    let mut pb = RichProgress::new(
        tqdm!(unit = " second".to_owned(), dynamic_ncols = true),
        vec![
            Column::Animation,
            Column::Percentage(2),
            Column::Text("•".to_owned()),
            Column::CountTotal,
            Column::Text("•".to_owned()),
            Column::ElapsedTime,
            Column::Text(">".to_owned()),
            Column::RemainingTime,
            Column::Text("•".to_owned()),
            Column::Text("[red]0 FPS".to_owned()),
        ],
    );

    let mut duration = None;
    let mut fps = None;
    let mut check_overwrite = true;
    let mut read_byte = b'\n';

    let duration_rx = Regex::new(r"Duration: (\d{2}):(\d{2}):(\d{2})\.\d{2}").unwrap();
    let fps_rx = Regex::new(r"(\d{2}\.\d{2}|\d{2}) fps").unwrap();
    let progress_rx = Regex::new(r"time=(\d{2}):(\d{2}):(\d{2})\.\d{2}").unwrap();

    loop {
        let mut prepend_text = "".to_owned();

        if check_overwrite {
            let mut pre_buf = [0; 5];
            reader
                .read_exact(&mut pre_buf)
                .map_err(|_| new_error("no such file or directory."))?;
            prepend_text.push_str(&String::from_utf8_lossy(&pre_buf));

            match prepend_text.as_str() {
                "File " => {
                    // File 'test.mp4' already exists. Overwrite? [y/N] y
                    let mut msg = String::new();

                    loop {
                        let mut post_buf = vec![];
                        reader.read_until(b']', &mut post_buf)?;
                        msg.push_str(&String::from_utf8(post_buf).unwrap());
                        let len = msg.len();

                        if 5 > len {
                            continue;
                        }

                        if msg
                            .get((len - 5)..len)
                            .map(|x| x == "[y/N]")
                            .unwrap_or(true)
                        {
                            break;
                        }
                    }

                    eprint!("File {} ", msg);
                    stderr().flush()?;
                    check_overwrite = false;
                    read_byte = b'\r';
                }

                "Press" => {
                    // Press [q] to stop, [?] for help
                    check_overwrite = false;
                    read_byte = b'\r';
                }

                _ => (),
            }
        } else {
            thread::sleep(Duration::from_secs_f32(0.1));
        }

        let mut buf = vec![];
        reader.read_until(read_byte, &mut buf)?;

        if let Ok(line) = String::from_utf8(buf) {
            let std_line = prepend_text + &line;

            if std_line == "" {
                pb.refresh()?;
                eprintln!();
                break;
            }

            if duration.is_none() {
                if let Some(x) = duration_rx.captures_iter(&std_line).next() {
                    duration = Some(
                        time_to_secs(&x)
                            .map_err(|_| new_error("couldn't parse total duration."))?,
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
                            .map_err(|_| new_error("couldn't parse fps."))?,
                    );
                    pb.pb.unit = " frame".to_owned();
                }
            }

            if let Some(x) = progress_rx.captures_iter(&std_line).next() {
                let mut current =
                    time_to_secs(&x).map_err(|_| new_error("couldn't parse current duration."))?;

                if let Some(frames) = fps {
                    current *= frames as usize;
                    if pb.pb.total == duration.unwrap_or(0) {
                        pb.pb.total *= frames as usize;
                    }
                }

                pb.replace(9, Column::Text(format!("[red]{:.0} FPS", pb.pb.rate())));

                if current >= pb.pb.total {
                    pb.write(std_line.replace("\r", "").replace("\n", ""))?;
                }

                pb.update_to(current)?;
            }
        } else {
            break;
        }
    }

    Ok(())
}

/*
ffmpeg version n6.1.1-1-g61b88b4dda-20240204 Copyright (c) 2000-2023 the FFmpeg developers
  built with gcc 13.2.0 (crosstool-NG 1.25.0.232_c175b21)
  configuration: --prefix=/ffbuild/prefix --pkg-config-flags=--static --pkg-config=pkg-config --cross-prefix=x86_64-w64-mingw32- --arch=x86_64 --target-os=mingw32 --enable-gpl --enable-version3 --disable-debug --enable-shared --disable-static --disable-w32threads --enable-pthreads --enable-iconv --enable-libxml2 --enable-zlib --enable-libfreetype --enable-libfribidi --enable-gmp --enable-lzma --enable-fontconfig --enable-libharfbuzz --enable-libvorbis --enable-opencl --disable-libpulse --enable-libvmaf --disable-libxcb --disable-xlib --enable-amf --enable-libaom --enable-libaribb24 --enable-avisynth --enable-chromaprint --enable-libdav1d --enable-libdavs2 --disable-libfdk-aac --enable-ffnvcodec --enable-cuda-llvm --enable-frei0r --enable-libgme --enable-libkvazaar --enable-libaribcaption --enable-libass --enable-libbluray --enable-libjxl --enable-libmp3lame --enable-libopus --enable-librist --enable-libssh --enable-libtheora --enable-libvpx --enable-libwebp --enable-lv2 --enable-libvpl --enable-openal --enable-libopencore-amrnb --enable-libopencore-amrwb --enable-libopenh264 --enable-libopenjpeg --enable-libopenmpt --enable-librav1e --enable-librubberband --enable-schannel --enable-sdl2 --enable-libsoxr --enable-libsrt --enable-libsvtav1 --enable-libtwolame --enable-libuavs3d --disable-libdrm --enable-vaapi --enable-libvidstab --enable-vulkan --enable-libshaderc --enable-libplacebo --enable-libx264 --enable-libx265 --enable-libxavs2 --enable-libxvid --enable-libzimg --enable-libzvbi --extra-cflags=-DLIBTWOLAME_STATIC --extra-cxxflags= --extra-ldflags=-pthread --extra-ldexeflags= --extra-libs=-lgomp --extra-version=20240204
  libavutil      58. 29.100 / 58. 29.100
  libavcodec     60. 31.102 / 60. 31.102
  libavformat    60. 16.100 / 60. 16.100
  libavdevice    60.  3.100 / 60.  3.100
  libavfilter     9. 12.100 /  9. 12.100
  libswscale      7.  5.100 /  7.  5.100
  libswresample   4. 12.100 /  4. 12.100
  libpostproc    57.  3.100 / 57.  3.100
Input #0, matroska,webm, from 'test.mkv':
  Metadata:
    encoder         : libebml v1.3.6 + libmatroska v1.4.9
    creation_time   : 2021-09-12T10:11:51.000000Z
    Encoded by      : ****
  Duration: 01:02:34.16, start: 0.000000, bitrate: 860 kb/s
  Stream #0:0: Video: hevc (Main 10), yuv420p10le(tv, bt709/bt709/unknown), 1280x720, SAR 1:1 DAR 16:9, 23.98 fps, 23.98 tbr, 1k tbn (default)
    Metadata:
      BPS-eng         : 777350
      DURATION-eng    : 01:02:33.542000000
      NUMBER_OF_FRAMES-eng: 89995
      NUMBER_OF_BYTES-eng: 364727328
      _STATISTICS_WRITING_APP-eng: mkvmerge v24.0.0 ('Beyond The Pale') 64-bit
      _STATISTICS_WRITING_DATE_UTC-eng: 2021-09-12 10:11:51
      _STATISTICS_TAGS-eng: BPS DURATION NUMBER_OF_FRAMES NUMBER_OF_BYTES
  Stream #0:1(kor): Audio: aac (LC), 48000 Hz, stereo, fltp
    Metadata:
      title           : Korean
      BPS-eng         : 80609
      DURATION-eng    : 01:02:34.155000000
      NUMBER_OF_FRAMES-eng: 175976
      NUMBER_OF_BYTES-eng: 37827792
      _STATISTICS_WRITING_APP-eng: mkvmerge v24.0.0 ('Beyond The Pale') 64-bit
      _STATISTICS_WRITING_DATE_UTC-eng: 2021-09-12 10:11:51
      _STATISTICS_TAGS-eng: BPS DURATION NUMBER_OF_FRAMES NUMBER_OF_BYTES
  Stream #0:2(eng): Subtitle: subrip
    Metadata:
      BPS-eng         : 60
      DURATION-eng    : 01:01:49.497000000
      NUMBER_OF_FRAMES-eng: 834
      NUMBER_OF_BYTES-eng: 27926
      _STATISTICS_WRITING_APP-eng: mkvmerge v24.0.0 ('Beyond The Pale') 64-bit
      _STATISTICS_WRITING_DATE_UTC-eng: 2021-09-12 10:11:51
      _STATISTICS_TAGS-eng: BPS DURATION NUMBER_OF_FRAMES NUMBER_OF_BYTES
  Stream #0:3(eng): Subtitle: subrip
    Metadata:
      BPS-eng         : 60
      DURATION-eng    : 01:01:49.497000000
      NUMBER_OF_FRAMES-eng: 834
      NUMBER_OF_BYTES-eng: 27926
      _STATISTICS_WRITING_APP-eng: mkvmerge v24.0.0 ('Beyond The Pale') 64-bit
      _STATISTICS_WRITING_DATE_UTC-eng: 2021-09-12 10:11:51
      _STATISTICS_TAGS-eng: BPS DURATION NUMBER_OF_FRAMES NUMBER_OF_BYTES
  Stream #0:4: Video: png, rgba(pc, gbr/unknown/unknown), 150x150 [SAR 2835:2835 DAR 1:1], 90k tbr, 90k tbn (attached pic)
    Metadata:
      filename        : cover.png
      mimetype        : image/png
File 'test.mp4' already exists. Overwrite? [y/N] y
Stream mapping:
  Stream #0:0 -> #0:0 (copy)
  Stream #0:1 -> #0:1 (aac (native) -> aac (native))
Press [q] to stop, [?] for help
Output #0, mp4, to 'test.mp4':
  Metadata:
    Encoded by      : ****
    encoder         : Lavf60.16.100
  Stream #0:0: Video: hevc (Main 10) (hev1 / 0x31766568), yuv420p10le(tv, bt709/bt709/unknown), 1280x720 [SAR 1:1 DAR 16:9], q=2-31, 23.98 fps, 23.98 tbr, 16k tbn (default)
    Metadata:
      BPS-eng         : 777350
      DURATION-eng    : 01:02:33.542000000
      NUMBER_OF_FRAMES-eng: 89995
      NUMBER_OF_BYTES-eng: 364727328
      _STATISTICS_WRITING_APP-eng: mkvmerge v24.0.0 ('Beyond The Pale') 64-bit
      _STATISTICS_WRITING_DATE_UTC-eng: 2021-09-12 10:11:51
      _STATISTICS_TAGS-eng: BPS DURATION NUMBER_OF_FRAMES NUMBER_OF_BYTES
  Stream #0:1(kor): Audio: aac (LC) (mp4a / 0x6134706D), 48000 Hz, stereo, fltp, 128 kb/s
    Metadata:
      title           : Korean
      BPS-eng         : 80609
      DURATION-eng    : 01:02:34.155000000
      NUMBER_OF_FRAMES-eng: 175976
      NUMBER_OF_BYTES-eng: 37827792
      _STATISTICS_WRITING_APP-eng: mkvmerge v24.0.0 ('Beyond The Pale') 64-bit
      _STATISTICS_WRITING_DATE_UTC-eng: 2021-09-12 10:11:51
      _STATISTICS_TAGS-eng: BPS DURATION NUMBER_OF_FRAMES NUMBER_OF_BYTES
      encoder         : Lavc60.31.102 aac
[out#0/mp4 @ 0000022163a68340] video:50451kB audio:8865kB subtitle:0kB other streams:0kB global headers:2kB muxing overhead: 0.967485%
size=   59890kB time=00:09:23.44 bitrate= 870.7kbits/s speed=60.9x
*/
