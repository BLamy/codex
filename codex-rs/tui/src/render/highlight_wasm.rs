use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;
use std::path::Path;
use std::sync::OnceLock;
use std::sync::RwLock;

const MAX_HIGHLIGHT_BYTES: usize = 512 * 1024;
const MAX_HIGHLIGHT_LINES: usize = 10_000;
const DEFAULT_THEME: &str = "catppuccin-mocha";

#[derive(Clone, Debug, Default)]
pub(crate) struct Theme {
    name: String,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct DiffScopeBackgroundRgbs {
    pub inserted: Option<(u8, u8, u8)>,
    pub deleted: Option<(u8, u8, u8)>,
}

pub(crate) struct ThemeEntry {
    pub name: String,
    pub is_custom: bool,
}

static THEME: OnceLock<RwLock<Theme>> = OnceLock::new();

fn theme_lock() -> &'static RwLock<Theme> {
    THEME.get_or_init(|| {
        RwLock::new(Theme {
            name: DEFAULT_THEME.to_string(),
        })
    })
}

pub(crate) fn set_theme_override(
    name: Option<String>,
    _codex_home: Option<std::path::PathBuf>,
) -> Option<String> {
    if let Some(name) = name {
        set_syntax_theme(Theme { name });
    }
    None
}

pub(crate) fn validate_theme_name(
    _name: Option<&str>,
    _codex_home: Option<&Path>,
) -> Option<String> {
    None
}

pub(crate) fn adaptive_default_theme_name() -> &'static str {
    DEFAULT_THEME
}

pub(crate) fn set_syntax_theme(theme: Theme) {
    let mut guard = match theme_lock().write() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    *guard = theme;
}

pub(crate) fn current_syntax_theme() -> Theme {
    match theme_lock().read() {
        Ok(theme) => theme.clone(),
        Err(poisoned) => poisoned.into_inner().clone(),
    }
}

pub(crate) fn diff_scope_background_rgbs() -> DiffScopeBackgroundRgbs {
    DiffScopeBackgroundRgbs::default()
}

pub(crate) fn foreground_style_for_scopes(_scope_names: &[&str]) -> Option<Style> {
    None
}

pub(crate) fn configured_theme_name() -> String {
    current_syntax_theme().name
}

pub(crate) fn resolve_theme_by_name(name: &str, _codex_home: Option<&Path>) -> Option<Theme> {
    Some(Theme {
        name: name.to_string(),
    })
}

pub(crate) fn list_available_themes(_codex_home: Option<&Path>) -> Vec<ThemeEntry> {
    [
        "ansi",
        "base16",
        "catppuccin-frappe",
        "catppuccin-latte",
        "catppuccin-macchiato",
        "catppuccin-mocha",
        "dracula",
        "github",
        "gruvbox-dark",
        "gruvbox-light",
        "nord",
        "one-half-dark",
        "one-half-light",
        "solarized-dark",
        "solarized-light",
    ]
    .into_iter()
    .map(|name| ThemeEntry {
        name: name.to_string(),
        is_custom: false,
    })
    .collect()
}

pub(crate) fn exceeds_highlight_limits(total_bytes: usize, total_lines: usize) -> bool {
    total_bytes > MAX_HIGHLIGHT_BYTES || total_lines > MAX_HIGHLIGHT_LINES
}

pub(crate) fn highlight_code_to_lines(code: &str, _lang: &str) -> Vec<Line<'static>> {
    if code.is_empty() {
        return vec![Line::from("")];
    }

    let mut lines: Vec<Line<'static>> = code
        .split('\n')
        .map(|line| Line::from(line.trim_end_matches('\r').to_string()))
        .collect();
    if code.ends_with('\n') {
        lines.pop();
    }
    lines
}

pub(crate) fn highlight_bash_to_lines(script: &str) -> Vec<Line<'static>> {
    highlight_code_to_lines(script, "bash")
}

pub(crate) fn highlight_code_to_styled_spans(
    code: &str,
    _lang: &str,
) -> Option<Vec<Vec<Span<'static>>>> {
    if code.is_empty() || exceeds_highlight_limits(code.len(), code.lines().count()) {
        return None;
    }
    Some(
        code.lines()
            .map(|line| vec![Span::raw(line.to_string())])
            .collect(),
    )
}
