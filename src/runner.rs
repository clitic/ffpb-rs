use crate::{
    Error,
    args::FfmpegArgs,
    progress::{ProgressBar, ProgressStats},
};
use std::{
    io::{self, BufRead, BufReader, Read, Write},
    process::{Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};

static INTERRUPTED: AtomicBool = AtomicBool::new(false);

fn parse_duration_line(line: &str) -> Option<f64> {
    // Example: "  Duration: 00:01:30.50, start: 0.000000, bitrate: 2450 kb/s"
    let marker = "Duration: ";
    let pos = line.find(marker)?;
    let rest = &line[pos + marker.len()..];
    let time_str = rest.split(',').next()?.trim();

    if time_str == "N/A" {
        return None;
    }

    crate::args::parse_time(time_str)
}

fn apply_progress_kv(stats: &mut ProgressStats, key: &str, value: &str) {
    match key {
        "frame" => {
            stats.frame = value.parse().unwrap_or(0);
        }
        "fps" => {
            stats.fps = value.parse().unwrap_or(0.0);
        }
        "stream_0_0_q" => {
            stats.q = value.parse().unwrap_or(0.0);
        }
        "bitrate" => {
            // e.g. "2450.3kbits/s" or "N/A"
            if let Some(kbits_str) = value.strip_suffix("kbits/s") {
                stats.bitrate_kbps = kbits_str.trim().parse().unwrap_or(0.0);
            }
        }
        "total_size" => {
            stats.total_size = value.parse().unwrap_or(0);
        }
        "out_time_us" => {
            stats.out_time_us = value.parse().unwrap_or(0);
        }
        "speed" => {
            // e.g. "1.82x" or "N/A"
            if let Some(speed_str) = value.strip_suffix('x') {
                stats.speed = speed_str.trim().parse().unwrap_or(0.0);
            }
        }
        "progress" => {
            stats.is_end = value == "end";
        }
        _ => {}
    }
}

/// Compute the effective output duration in microseconds.
pub fn compute_effective_duration(
    args: &FfmpegArgs,
    total_duration_secs: Option<f64>,
) -> Option<u64> {
    let ss = args.ss.unwrap_or(0.0);

    let effective = match (args.to, args.t, total_duration_secs) {
        // -ss and -to: effective = to - ss
        (Some(to), _, _) => {
            let eff = to - ss;
            if eff > 0.0 { Some(eff) } else { None }
        }
        // -ss and -t: effective = t
        (_, Some(t), _) => Some(t),
        // -ss only: effective = total - ss
        (None, None, Some(total)) => {
            let eff = total - ss;
            if eff > 0.0 { Some(eff) } else { None }
        }
        // No flags, just total duration
        (None, None, None) => None,
    };

    effective.map(|secs| (secs * 1_000_000.0) as u64)
}

pub fn run_ffmpeg(args: &FfmpegArgs) -> Result<i32, Error> {
    let _ = ctrlc::set_handler(|| {
        INTERRUPTED.store(true, Ordering::SeqCst);
    });

    let mut child = Command::new("ffmpeg")
        .args(&args.args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            if e.kind() == io::ErrorKind::NotFound {
                Error::FfmpegNotFound
            } else {
                Error::SpawnFailed(e)
            }
        })?;

    // Shared storage for the duration parsed from stderr
    let duration_secs = Arc::new(Mutex::new(None));

    // In clean mode: discard stderr output entirely.
    // In default mode: forward stderr before encoding starts, suppress during encoding.
    let clean_mode = args.clean;
    let encoding_active = Arc::new(AtomicBool::new(false));
    let stderr_buffer = Arc::new(Mutex::new(Vec::<u8>::new()));

    let mut stderr = child.stderr.take().expect("stderr should be piped");
    let duration_clone = Arc::clone(&duration_secs);
    let encoding_clone = Arc::clone(&encoding_active);
    let buffer_clone = Arc::clone(&stderr_buffer);
    let stderr_handle = thread::spawn(move || {
        let real_stderr = io::stderr();
        let mut found_duration = false;
        let mut line_buf = String::new();
        let mut buf = [0u8; 256];

        loop {
            match stderr.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    let chunk = &buf[..n];

                    if !clean_mode {
                        if encoding_clone.load(Ordering::SeqCst) {
                            if let Ok(mut buffer) = buffer_clone.lock() {
                                buffer.extend_from_slice(chunk);
                            }
                        } else {
                            let mut stderr = real_stderr.lock();
                            let _ = stderr.write_all(chunk);
                            let _ = stderr.flush();
                        }
                    }

                    // Parse duration internally
                    if !found_duration && let Ok(s) = std::str::from_utf8(chunk) {
                        line_buf.push_str(s);
                        while let Some(pos) = line_buf.find('\n') {
                            let line = line_buf[..pos].to_string();
                            line_buf.drain(..=pos);
                            if let Some(dur) = parse_duration_line(&line) {
                                if let Ok(mut lock) = duration_clone.lock() {
                                    *lock = Some(dur);
                                }
                                found_duration = true;
                                break;
                            }
                        }
                    }
                }
            }
        }
    });

    // Read progress from stdout
    let stdout = child.stdout.take().expect("stdout should be piped");
    let reader = BufReader::new(stdout);

    let mut progress_bar: Option<ProgressBar> = None;
    let mut stats = ProgressStats::default();
    let mut bar_initialized = false;

    for line in reader.lines() {
        if INTERRUPTED.load(Ordering::SeqCst) {
            if let Some(ref mut bar) = progress_bar {
                bar.interrupt();
            }
            let _ = child.wait();
            return Ok(130);
        }

        let Ok(line) = line else { break };
        let line = line.trim();

        if let Some((key, value)) = line.split_once('=') {
            apply_progress_kv(&mut stats, key.trim(), value.trim());

            if key.trim() == "progress" {
                if !bar_initialized {
                    encoding_active.store(true, Ordering::SeqCst);
                    let total_dur = duration_secs.lock().ok().and_then(|d| *d);
                    let effective_us = compute_effective_duration(args, total_dur);
                    progress_bar = Some(ProgressBar::new(effective_us, clean_mode));
                    bar_initialized = true;
                }

                if let Some(ref mut bar) = progress_bar {
                    if let Ok(lock) = duration_secs.lock()
                        && let Some(dur) = *lock
                        && let Some(eff) = compute_effective_duration(args, Some(dur))
                    {
                        bar.set_total_duration(eff);
                    }

                    if stats.is_end {
                        bar.finish(&stats);
                    } else {
                        bar.update(&stats, false);
                    }
                }

                if stats.is_end {
                    break;
                }

                stats.is_end = false;
            }
        }
    }

    // Wait for stderr thread
    let _ = stderr_handle.join();

    // Flush buffered stderr from encoding phase
    if !clean_mode {
        if let Ok(buffer) = stderr_buffer.lock() {
            if !buffer.is_empty() {
                let stderr = io::stderr();
                let mut lock = stderr.lock();
                let _ = lock.write_all(&buffer);
                let _ = lock.flush();
            }
        }
    }

    // Wait for ffmpeg to exit
    let status = child.wait().map_err(Error::SpawnFailed)?;

    Ok(status.code().unwrap_or(1))
}
