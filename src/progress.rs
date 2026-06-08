use std::{
    fmt::Write as FmtWrite,
    io::{self, Write},
    time::Instant,
};

const BAR_WIDTH: usize = 40;
const PB_START: (u8, u8, u8) = (168, 85, 247);
const PB_END: (u8, u8, u8) = (236, 72, 153);
const DIM_COLOR: (u8, u8, u8) = (55, 65, 81);
const DONE_COLOR: (u8, u8, u8) = (75, 181, 67);

fn fg(buf: &mut String, r: u8, g: u8, b: u8) {
    let _ = write!(buf, "\x1b[38;2;{r};{g};{b}m");
}

fn bold(buf: &mut String) {
    buf.push_str("\x1b[1m");
}

fn dim(buf: &mut String) {
    buf.push_str("\x1b[2m");
}

fn reset(buf: &mut String) {
    buf.push_str("\x1b[0m");
}

fn lerp_color(t: f64, from: (u8, u8, u8), to: (u8, u8, u8)) -> (u8, u8, u8) {
    let r = from.0 as f64 + (to.0 as f64 - from.0 as f64) * t;
    let g = from.1 as f64 + (to.1 as f64 - from.1 as f64) * t;
    let b = from.2 as f64 + (to.2 as f64 - from.2 as f64) * t;
    (r as u8, g as u8, b as u8)
}

fn format_size(bytes: u64) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = 1024.0 * 1024.0;
    const GIB: f64 = 1024.0 * 1024.0 * 1024.0;

    let b = bytes as f64;
    if b >= GIB {
        format!("{:.1} GiB", b / GIB)
    } else if b >= MIB {
        format!("{:.1} MiB", b / MIB)
    } else if b >= KIB {
        format!("{:.1} KiB", b / KIB)
    } else {
        format!("{bytes} B")
    }
}

pub fn format_time(ms: u64) -> String {
    let total_secs = ms / 1_000;
    let hours = total_secs / 3600;
    let mins = (total_secs % 3600) / 60;
    let secs = total_secs % 60;

    if hours > 0 {
        format!("{hours}h{mins}m{secs}s")
    } else if mins > 0 {
        format!("{mins}m{secs}s")
    } else {
        format!("{secs}s")
    }
}

pub fn format_time_clock(ms: u64) -> String {
    let total_secs = ms / 1_000;
    let hours = total_secs / 3600;
    let mins = (total_secs % 3600) / 60;
    let secs = total_secs % 60;

    if hours > 0 {
        format!("{hours:02}:{mins:02}:{secs:02}")
    } else {
        format!("{mins:02}:{secs:02}")
    }
}

#[derive(Default)]
pub struct ProgressStats {
    pub frame: u64,
    pub fps: f64,
    pub bitrate_kbps: f64,
    pub total_size: u64,
    pub out_time_ms: u64,
    pub speed: f64,
    pub q: f64,
    pub is_end: bool,
}

pub struct ProgressBar {
    total_duration_ms: Option<u64>,
    last_render: Option<Instant>,
    started_at: Instant,
    lines_rendered: usize,
    pulse_frame: usize,
    compact: bool,
}

impl ProgressBar {
    pub fn new(total_duration_ms: Option<u64>, compact: bool) -> Self {
        eprint!("\x1b[?25l");
        Self {
            total_duration_ms,
            last_render: None,
            started_at: Instant::now(),
            lines_rendered: 0,
            pulse_frame: 0,
            compact,
        }
    }

    pub fn set_total_duration(&mut self, ms: u64) {
        self.total_duration_ms = Some(ms);
    }

    pub fn update(&mut self, stats: &ProgressStats, force: bool) {
        if !force
            && let Some(last) = self.last_render
            && last.elapsed().as_millis() < 1000
        {
            return;
        }
        self.last_render = Some(Instant::now());
        self.pulse_frame = self.pulse_frame.wrapping_add(1);
        self.render(stats, false);
    }

    pub fn finish(&mut self, stats: &ProgressStats) {
        self.render(stats, true);
        self.lines_rendered = 0;
        eprint!("\x1b[?25h");
        let _ = io::stderr().flush();
    }

    pub fn interrupt(&mut self) {
        self.clear_lines();
        eprint!("\x1b[?25h");
        let _ = io::stderr().flush();
    }

    fn clear_lines(&self) {
        let mut stderr = io::stderr().lock();
        let _ = write!(stderr, "\x1b[2K");
        for _ in 1..self.lines_rendered {
            let _ = write!(stderr, "\x1b[A\x1b[2K");
        }
        let _ = write!(stderr, "\r");
        let _ = stderr.flush();
    }

    fn render(&mut self, stats: &ProgressStats, finished: bool) {
        self.clear_lines();
        let mut buf = String::with_capacity(100);
        let indent = if self.compact { "" } else { "  " };

        buf.push_str(indent);

        if finished {
            fg(&mut buf, DONE_COLOR.0, DONE_COLOR.1, DONE_COLOR.2);
            bold(&mut buf);
            buf.push_str("Done");
            reset(&mut buf);
        } else {
            bold(&mut buf);
            buf.push_str("Encoded");
            reset(&mut buf);

            fg(&mut buf, PB_START.0, PB_START.1, PB_START.2);
            let _ = write!(buf, " {}", format_time_clock(stats.out_time_ms));
            reset(&mut buf);

            if let Some(total) = self.total_duration_ms {
                let _ = write!(buf, "/{}", format_time_clock(total));
            }

            let elapsed_ms = self.started_at.elapsed().as_millis() as u64;
            let _ = write!(buf, " in {}", format_time(elapsed_ms));
        }

        buf.push('\n');
        buf.push_str(indent);

        let progress_fraction = if let Some(total) = self.total_duration_ms && total > 0 {
            let frac = stats.out_time_ms as f64 / total as f64;
            if finished {
                1.0
            } else {
                frac.min(1.0)
            }
        } else if finished {
            1.0
        } else {
            0.0
        };

        if self.total_duration_ms.is_none() && !finished {
            let pulse_width = 7;
            let cycle = (self.pulse_frame * 3) % (BAR_WIDTH + pulse_width);

            for i in 0..BAR_WIDTH {
                let in_pulse = i >= cycle.saturating_sub(pulse_width) && i < cycle;

                if in_pulse {
                    let t = (i as f64) / (BAR_WIDTH as f64);
                    let (r, g, b) = lerp_color(t, PB_START, PB_END);
                    fg(&mut buf, r, g, b);
                    buf.push('█');
                } else {
                    fg(&mut buf, DIM_COLOR.0, DIM_COLOR.1, DIM_COLOR.2);
                    buf.push('░');
                }
            }
            reset(&mut buf);
        } else {
            let filled = (progress_fraction * BAR_WIDTH as f64).round() as usize;
            let filled = filled.min(BAR_WIDTH);

            for i in 0..BAR_WIDTH {
                if i < filled {
                    let t = i as f64 / (BAR_WIDTH - 1) as f64;
                    let color = if finished {
                        DONE_COLOR
                    } else {
                        lerp_color(t, PB_START, PB_END)
                    };
                    fg(&mut buf, color.0, color.1, color.2);
                    buf.push('█');
                } else {
                    fg(&mut buf, DIM_COLOR.0, DIM_COLOR.1, DIM_COLOR.2);
                    buf.push('░');
                }
            }
            reset(&mut buf);

            bold(&mut buf);
            let _ = write!(buf, " {:.1}%", progress_fraction * 100.0);
            reset(&mut buf);

            if !finished
                && let Some(total) = self.total_duration_ms
                && stats.out_time_ms > 0
                && stats.out_time_ms < total
            {
                let elapsed_ms = self.started_at.elapsed().as_millis() as u64;
                let remaining_ms = total.saturating_sub(stats.out_time_ms);
                let eta_ms = (elapsed_ms as f64 * remaining_ms as f64 / stats.out_time_ms as f64)
                    as u64;
                dim(&mut buf);
                buf.push_str(" • ");
                reset(&mut buf);
                fg(&mut buf, PB_START.0, PB_START.1, PB_START.2);
                let _ = write!(buf, "eta {}", format_time(eta_ms));
                reset(&mut buf);
            }
        }

        buf.push('\n');
        buf.push_str(indent);

        let _ = write!(buf, "{}", stats.frame);
        dim(&mut buf);
        buf.push_str(" @ ");
        reset(&mut buf);
        let _ = write!(buf, "{:.1} fps", stats.fps);
        dim(&mut buf);
        buf.push_str(" • ");
        reset(&mut buf);
        let _ = write!(buf, "{:.1}q", stats.q);
        dim(&mut buf);
        buf.push_str(" • ");
        reset(&mut buf);
        let _ = write!(buf, "{}", format_size(stats.total_size));
        dim(&mut buf);
        buf.push_str(" • ");
        reset(&mut buf);
        let _ = write!(buf, "{:.1} kbps", stats.bitrate_kbps);
        dim(&mut buf);
        buf.push_str(" • ");
        reset(&mut buf);
        let _ = write!(buf, "{:.1}x", stats.speed);

        self.lines_rendered = 3;

        if finished {
            buf.push('\n');
        }

        let mut stderr = io::stderr().lock();
        let _ = write!(stderr, "{buf}");
        let _ = stderr.flush();
    }
}

impl Drop for ProgressBar {
    fn drop(&mut self) {
        eprint!("\x1b[?25h");
        let _ = io::stderr().flush();
    }
}
