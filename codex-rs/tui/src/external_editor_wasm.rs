use std::env;

use color_eyre::eyre::Report;
use color_eyre::eyre::Result;
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum EditorError {
    #[error("neither VISUAL nor EDITOR is set")]
    MissingEditor,
    #[error("failed to parse editor command")]
    ParseFailed,
    #[error("editor command is empty")]
    EmptyCommand,
}

pub(crate) fn resolve_editor_command() -> std::result::Result<Vec<String>, EditorError> {
    let raw = env::var("VISUAL")
        .or_else(|_| env::var("EDITOR"))
        .map_err(|_| EditorError::MissingEditor)?;
    let parts = shlex::split(&raw).ok_or(EditorError::ParseFailed)?;
    if parts.is_empty() {
        return Err(EditorError::EmptyCommand);
    }
    Ok(parts)
}

pub(crate) async fn run_editor(_seed: &str, _editor_cmd: &[String]) -> Result<String> {
    Err(Report::msg(
        "external editor is unavailable in the browser Codex TUI",
    ))
}
