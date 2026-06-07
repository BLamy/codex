//! Rollout persistence and discovery for Codex session files.

#[cfg(not(target_arch = "wasm32"))]
use std::sync::LazyLock;

#[cfg(not(target_arch = "wasm32"))]
use codex_protocol::protocol::SessionSource;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod compression;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod config;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod list;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod metadata;
#[cfg(not(target_arch = "wasm32"))]
mod persistence_metrics;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod policy;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod recorder;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod search;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod session_index;
#[cfg(not(target_arch = "wasm32"))]
mod sqlite_metrics;
#[cfg(not(target_arch = "wasm32"))]
pub mod state_db;
#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod default_client {
    pub use codex_login::default_client::*;
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) use codex_protocol::protocol;

#[cfg(not(target_arch = "wasm32"))]
pub const SESSIONS_SUBDIR: &str = "sessions";
#[cfg(not(target_arch = "wasm32"))]
pub const ARCHIVED_SESSIONS_SUBDIR: &str = "archived_sessions";
#[cfg(not(target_arch = "wasm32"))]
pub static INTERACTIVE_SESSION_SOURCES: LazyLock<Vec<SessionSource>> = LazyLock::new(|| {
    vec![
        SessionSource::Cli,
        SessionSource::VSCode,
        SessionSource::Custom("atlas".to_string()),
        SessionSource::Custom("chatgpt".to_string()),
    ]
});

#[cfg(not(target_arch = "wasm32"))]
pub use codex_protocol::protocol::SessionMeta;
#[cfg(not(target_arch = "wasm32"))]
pub use compression::RolloutLineReader;
#[cfg(not(target_arch = "wasm32"))]
pub use compression::existing_rollout_path;
#[cfg(not(target_arch = "wasm32"))]
pub use compression::open_rollout_line_reader;
#[cfg(not(target_arch = "wasm32"))]
pub use compression::plain_rollout_path;
#[cfg(not(target_arch = "wasm32"))]
pub use compression::spawn_rollout_compression_worker;
#[cfg(not(target_arch = "wasm32"))]
pub use config::Config;
#[cfg(not(target_arch = "wasm32"))]
pub use config::RolloutConfig;
#[cfg(not(target_arch = "wasm32"))]
pub use config::RolloutConfigView;
#[cfg(not(target_arch = "wasm32"))]
pub use list::Cursor;
#[cfg(not(target_arch = "wasm32"))]
pub use list::SortDirection;
#[cfg(not(target_arch = "wasm32"))]
pub use list::ThreadItem;
#[cfg(not(target_arch = "wasm32"))]
pub use list::ThreadListConfig;
#[cfg(not(target_arch = "wasm32"))]
pub use list::ThreadListLayout;
#[cfg(not(target_arch = "wasm32"))]
pub use list::ThreadSortKey;
#[cfg(not(target_arch = "wasm32"))]
pub use list::ThreadsPage;
#[cfg(not(target_arch = "wasm32"))]
pub use list::find_archived_thread_path_by_id_str;
#[cfg(not(target_arch = "wasm32"))]
pub use list::find_thread_path_by_id_str;
#[cfg(not(target_arch = "wasm32"))]
#[deprecated(note = "use find_thread_path_by_id_str")]
pub use list::find_thread_path_by_id_str as find_conversation_path_by_id_str;
#[cfg(not(target_arch = "wasm32"))]
pub use list::get_threads;
#[cfg(not(target_arch = "wasm32"))]
pub use list::get_threads_in_root;
#[cfg(not(target_arch = "wasm32"))]
pub use list::parse_cursor;
#[cfg(not(target_arch = "wasm32"))]
pub use list::read_head_for_summary;
#[cfg(not(target_arch = "wasm32"))]
pub use list::read_session_meta_line;
#[cfg(not(target_arch = "wasm32"))]
pub use list::read_thread_item_from_rollout;
#[cfg(not(target_arch = "wasm32"))]
pub use list::rollout_date_parts;
#[cfg(not(target_arch = "wasm32"))]
pub use metadata::builder_from_items;
#[cfg(not(target_arch = "wasm32"))]
pub use persistence_metrics::RolloutPersistenceBatchMeasurement;
#[cfg(not(target_arch = "wasm32"))]
pub use persistence_metrics::RolloutPersistenceTelemetry;
#[cfg(not(target_arch = "wasm32"))]
pub use persistence_metrics::measure_and_filter_rollout_items;
#[cfg(not(target_arch = "wasm32"))]
pub use policy::is_persisted_rollout_item;
#[cfg(not(target_arch = "wasm32"))]
pub use policy::persisted_rollout_items;
#[cfg(not(target_arch = "wasm32"))]
pub use policy::should_persist_response_item_for_memories;
#[cfg(not(target_arch = "wasm32"))]
pub use recorder::RolloutRecorder;
#[cfg(not(target_arch = "wasm32"))]
pub use recorder::RolloutRecorderParams;
#[cfg(not(target_arch = "wasm32"))]
pub use recorder::append_rollout_item_to_path;
#[cfg(not(target_arch = "wasm32"))]
pub use search::first_rollout_content_match_snippet;
#[cfg(not(target_arch = "wasm32"))]
pub use search::search_rollout_matches;
#[cfg(not(target_arch = "wasm32"))]
pub use search::search_rollout_paths;
#[cfg(not(target_arch = "wasm32"))]
pub use session_index::append_thread_name;
#[cfg(not(target_arch = "wasm32"))]
pub use session_index::find_thread_meta_by_name_str;
#[cfg(not(target_arch = "wasm32"))]
pub use session_index::find_thread_name_by_id;
#[cfg(not(target_arch = "wasm32"))]
pub use session_index::find_thread_names_by_ids;
#[cfg(not(target_arch = "wasm32"))]
pub use session_index::remove_thread_name_entries;
#[cfg(not(target_arch = "wasm32"))]
pub use state_db::StateDbHandle;
#[cfg(not(target_arch = "wasm32"))]
pub use state_db::sqlite_telemetry_recorder;
#[cfg(target_arch = "wasm32")]
pub use wasm::*;

#[cfg(test)]
mod tests;
