pub struct FfmpegArgs {
    pub ss: Option<f64>,
    pub to: Option<f64>,
    pub t: Option<f64>,
    pub clean: bool,
    pub args: Vec<String>,
}

pub fn parse_time(s: &str) -> Option<f64> {
    let parts = s.split(':').collect::<Vec<_>>();
    match parts.len() {
        1 => parts[0].parse::<f64>().ok(),
        2 => {
            let mins = parts[0].parse::<f64>().ok()?;
            let secs = parts[1].parse::<f64>().ok()?;
            Some(mins * 60.0 + secs)
        }
        3 => {
            let hours = parts[0].parse::<f64>().ok()?;
            let mins = parts[1].parse::<f64>().ok()?;
            let secs = parts[2].parse::<f64>().ok()?;
            Some(hours * 3600.0 + mins * 60.0 + secs)
        }
        _ => None,
    }
}

pub fn parse_args(args: &[String]) -> FfmpegArgs {
    let mut ss = None;
    let mut to = None;
    let mut t = None;
    let mut clean = false;
    let mut has_progress = false;
    let mut has_nostats = false;

    let mut iter = args.iter().peekable();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-ss" => {
                if let Some(val) = iter.peek() {
                    ss = parse_time(val);
                }
            }
            "-to" => {
                if let Some(val) = iter.peek() {
                    to = parse_time(val);
                }
            }
            "-t" => {
                if let Some(val) = iter.peek() {
                    t = parse_time(val);
                }
            }
            "-progress" => {
                has_progress = true;
            }
            "-nostats" => {
                has_nostats = true;
            }
            "--clean" => {
                clean = true;
            }
            _ => {}
        }
    }

    let mut args = args
        .iter()
        .filter(|a| *a != "--clean")
        .cloned()
        .collect::<Vec<_>>();

    if !has_progress {
        args.push("-progress".to_string());
        args.push("pipe:1".to_string());
    }
    if !has_nostats {
        args.push("-nostats".to_string());
    }

    FfmpegArgs {
        ss,
        to,
        t,
        clean,
        args,
    }
}
