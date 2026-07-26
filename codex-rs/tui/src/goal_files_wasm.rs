//! Browser-safe goal draft surface.
//!
//! Materialization is performed by the app-server after the draft crosses the
//! browser host boundary, so the TUI only owns the current 0.145 draft shape.

use crate::bottom_pane::LocalImageAttachment;
use codex_protocol::user_input::TextElement;

#[derive(Clone, Debug, Default)]
pub(crate) struct GoalDraft {
    pub(crate) objective: String,
    pub(crate) text_elements: Vec<TextElement>,
    pub(crate) pending_pastes: Vec<(String, String)>,
    pub(crate) local_images: Vec<LocalImageAttachment>,
    pub(crate) remote_image_urls: Vec<String>,
}
