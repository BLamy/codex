use codex_tui::browser::BrowserTuiFrame;
use codex_tui::browser::BrowserTuiStatus;
use codex_tui::browser::DEFAULT_HEIGHT;
use codex_tui::browser::DEFAULT_MODEL;
use codex_tui::browser::DEFAULT_WIDTH;
use codex_tui::browser::browser_tui_version;
use codex_tui::browser::render_browser_tui_frame_to_ansi;

use crate::browser_cli::BrowserRunOptions;
use crate::browser_cli::BrowserRunResult;

pub(crate) fn render_browser_tui_frame(
    args: Vec<String>,
    options: BrowserRunOptions,
) -> BrowserRunResult {
    let frame = match browser_tui_frame_from_args(args, options) {
        Ok(frame) => frame,
        Err(message) => return BrowserRunResult::stderr(message, 2),
    };

    match render_browser_tui_frame_to_ansi(&frame) {
        Ok(ansi) => BrowserRunResult::stdout(ansi, 0),
        Err(message) => BrowserRunResult::stderr(message, 1),
    }
}

fn browser_tui_frame_from_args(
    args: Vec<String>,
    options: BrowserRunOptions,
) -> Result<BrowserTuiFrame, String> {
    let mut width = DEFAULT_WIDTH;
    let mut height = DEFAULT_HEIGHT;
    let mut version = browser_tui_version();
    let mut model = DEFAULT_MODEL.to_string();
    let mut directory = options.cwd.unwrap_or_else(|| "~".to_string());
    let mut input = String::new();
    let mut placeholder = Some("Explain this code base to me".to_string());
    let mut status = BrowserTuiStatus::Ready;
    let mut transcript = Vec::new();

    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--width" => {
                width = parse_dimension("--width", iter.next())?;
            }
            "--height" => {
                height = parse_dimension("--height", iter.next())?;
            }
            "--model" => {
                model = iter
                    .next()
                    .ok_or_else(|| "--model requires a value\n".to_string())?;
            }
            "--version" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "--version requires a value\n".to_string())?;
                version = (!value.is_empty() && value != "0.0.0").then_some(value);
            }
            "--directory" | "--cwd" => {
                directory = iter
                    .next()
                    .ok_or_else(|| "--directory requires a value\n".to_string())?;
            }
            "--input" => {
                input = iter
                    .next()
                    .ok_or_else(|| "--input requires a value\n".to_string())?;
            }
            "--placeholder" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "--placeholder requires a value\n".to_string())?;
                placeholder = (!value.is_empty()).then_some(value);
            }
            "--status" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "--status requires a value\n".to_string())?;
                status = BrowserTuiStatus::parse(&value)?;
            }
            "--transcript" => {
                transcript.push(
                    iter.next()
                        .ok_or_else(|| "--transcript requires a value\n".to_string())?,
                );
            }
            _ => {
                return Err(format!("unrecognized browser TUI frame argument: {arg}\n"));
            }
        }
    }

    Ok(BrowserTuiFrame {
        width: width.clamp(40, 240),
        height: height.clamp(12, 80),
        version,
        model,
        directory,
        input,
        placeholder,
        status,
        transcript,
    })
}

fn parse_dimension(name: &str, value: Option<String>) -> Result<u16, String> {
    value
        .ok_or_else(|| format!("{name} requires a value\n"))?
        .parse::<u16>()
        .map_err(|_| format!("{name} must be an integer\n"))
}
