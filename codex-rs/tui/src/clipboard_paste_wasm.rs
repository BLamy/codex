use std::path::Path;
use std::path::PathBuf;

use crate::clipboard_paste_wasm::EncodedImageFormat::Other;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodedImageFormat {
    Png,
    Jpeg,
    Other,
}

impl EncodedImageFormat {
    pub fn label(self) -> &'static str {
        match self {
            EncodedImageFormat::Png => "PNG",
            EncodedImageFormat::Jpeg => "JPEG",
            EncodedImageFormat::Other => "IMG",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PastedImageInfo {
    pub width: u32,
    pub height: u32,
    pub encoded_format: EncodedImageFormat,
}

pub(crate) fn normalize_pasted_path(text: &str) -> Option<PathBuf> {
    let trimmed = text.trim();
    if trimmed.is_empty() || trimmed.contains('\n') {
        return None;
    }
    Some(PathBuf::from(trimmed))
}

pub(crate) fn normalize_pasted_search_query(text: &str) -> Option<String> {
    let trimmed = text.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

pub(crate) fn paste_image_to_temp_png() -> Result<(PathBuf, PastedImageInfo), String> {
    Err("image clipboard paste is unavailable in the browser Codex TUI".to_string())
}

pub(crate) fn pasted_image_format(path: &Path) -> EncodedImageFormat {
    match path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("png") => EncodedImageFormat::Png,
        Some("jpg" | "jpeg") => EncodedImageFormat::Jpeg,
        _ => Other,
    }
}
