// Forbid accidental stdout/stderr writes in the *library* portion of the TUI.
// The standalone `codex-tui` binary prints a short help message before the
// alternate-screen mode starts; that file opts-out locally via `allow`.
#![deny(clippy::print_stdout, clippy::print_stderr)]
#![deny(clippy::disallowed_methods)]

pub mod browser;
mod time;

#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
extern crate self as codex_app_server_protocol;

#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
extern crate self as codex_connectors;

#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
extern crate self as codex_core_skills;

#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
extern crate self as codex_file_search;

#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
extern crate self as codex_plugin;

#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
extern crate self as image;

#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
#[path = "codex_app_server_protocol_wasm.rs"]
mod codex_app_server_protocol_wasm;

#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
pub use codex_app_server_protocol_wasm::AppInfo;

#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
#[path = "codex_connectors_wasm.rs"]
mod codex_connectors_wasm;

#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
pub mod metadata {
    pub use crate::codex_connectors_wasm::metadata::*;
}

#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
#[path = "codex_core_skills_wasm.rs"]
mod codex_core_skills_wasm;

#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
pub mod model {
    pub use crate::codex_core_skills_wasm::model::*;
}

#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
#[path = "codex_file_search_wasm.rs"]
mod codex_file_search_wasm;

#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
pub use codex_file_search_wasm::FileMatch;
#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
pub use codex_file_search_wasm::MatchType;

#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
#[path = "codex_plugin_wasm.rs"]
mod codex_plugin_wasm;

#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
pub use codex_plugin_wasm::AppConnectorId;
#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
pub use codex_plugin_wasm::PluginCapabilitySummary;

#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
pub fn image_dimensions(_path: impl AsRef<std::path::Path>) -> Result<(u32, u32), &'static str> {
    Err("image dimensions are unavailable in the browser TUI")
}

#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
#[path = "app_event_wasm.rs"]
mod app_event;

#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
#[path = "app_event_sender_wasm.rs"]
mod app_event_sender;

#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
#[path = "bottom_pane/wasm.rs"]
pub(crate) mod bottom_pane;

#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
mod color;

#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
pub(crate) mod custom_terminal;

#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
mod clipboard_paste_wasm;

#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
use clipboard_paste_wasm as clipboard_paste;

#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
#[path = "history_cell_wasm.rs"]
mod history_cell;

#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
mod key_hint;

#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
#[path = "keymap_wasm.rs"]
mod keymap;

#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
mod line_truncation;

#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
mod markdown;

#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
mod markdown_render;

#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
#[path = "render_wasm.rs"]
mod render;

#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
#[path = "mention_codec_wasm.rs"]
mod mention_codec;

#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
#[path = "session_log_wasm.rs"]
mod session_log;

#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
#[path = "skills_helpers_wasm.rs"]
mod skills_helpers;

#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
mod slash_command;

#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
#[path = "status_wasm.rs"]
mod status;

#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
#[path = "style_wasm.rs"]
mod style;

#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
mod text_formatting;

#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
mod terminal_hyperlinks;

#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
mod table_detect;

#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
mod tui_wasm;

#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
use tui_wasm as tui;

#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
mod ui_consts;

#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
mod wrapping;

#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
mod width;

#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
mod onboarding_wasm {
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;

    pub(crate) fn mark_underlined_hyperlink(_buf: &mut Buffer, _area: Rect, _url: &str) {}
}

#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
use onboarding_wasm as onboarding;

#[cfg(not(target_arch = "wasm32"))]
include!("lib_native.rs");

#[cfg(all(target_arch = "wasm32", feature = "real-tui-wasm"))]
include!("lib_wasm_real.rs");
