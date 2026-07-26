//! Rollout persistence and discovery for Codex session files.

use std::sync::LazyLock;

use codex_protocol::protocol::SessionSource;

#[cfg(target_arch = "wasm32")]
mod browser;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod compression;
pub(crate) mod config;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod list;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod metadata;
mod model_context;
#[cfg(not(target_arch = "wasm32"))]
mod ordinal;
mod persistence_metrics;
pub(crate) mod policy;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod recorder;
mod reverse_jsonl_scanner;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod search;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod session_index;
#[cfg(not(target_arch = "wasm32"))]
mod sqlite_metrics;
#[cfg(not(target_arch = "wasm32"))]
pub mod state_db;

pub(crate) use codex_protocol::protocol;

pub const SESSIONS_SUBDIR: &str = "sessions";
pub const ARCHIVED_SESSIONS_SUBDIR: &str = "archived_sessions";
pub static INTERACTIVE_SESSION_SOURCES: LazyLock<Vec<SessionSource>> = LazyLock::new(|| {
    vec![
        SessionSource::Cli,
        SessionSource::VSCode,
        SessionSource::Custom("atlas".to_string()),
        SessionSource::Custom("chatgpt".to_string()),
    ]
});

#[cfg(target_arch = "wasm32")]
pub use browser::Cursor;
#[cfg(target_arch = "wasm32")]
pub use browser::RolloutLineReader;
#[cfg(target_arch = "wasm32")]
pub use browser::RolloutRecorder;
#[cfg(target_arch = "wasm32")]
pub use browser::RolloutRecorderParams;
#[cfg(target_arch = "wasm32")]
pub use browser::SortDirection;
#[cfg(target_arch = "wasm32")]
pub use browser::StateDbHandle;
#[cfg(target_arch = "wasm32")]
pub use browser::ThreadItem;
#[cfg(target_arch = "wasm32")]
pub use browser::ThreadListConfig;
#[cfg(target_arch = "wasm32")]
pub use browser::ThreadListLayout;
#[cfg(target_arch = "wasm32")]
pub use browser::ThreadSortKey;
#[cfg(target_arch = "wasm32")]
pub use browser::ThreadsPage;
#[cfg(target_arch = "wasm32")]
pub use browser::append_rollout_item_to_path;
#[cfg(target_arch = "wasm32")]
pub use browser::append_thread_name;
#[cfg(target_arch = "wasm32")]
pub use browser::builder_from_items;
#[cfg(target_arch = "wasm32")]
pub use browser::existing_rollout_path;
#[cfg(target_arch = "wasm32")]
pub use browser::find_archived_thread_path_by_id_str;
#[cfg(target_arch = "wasm32")]
pub use browser::find_thread_meta_by_name_str;
#[cfg(target_arch = "wasm32")]
pub use browser::find_thread_name_by_id;
#[cfg(target_arch = "wasm32")]
pub use browser::find_thread_names_by_ids;
#[cfg(target_arch = "wasm32")]
pub use browser::find_thread_path_by_id_str;
#[cfg(target_arch = "wasm32")]
#[deprecated(note = "use find_thread_path_by_id_str")]
pub use browser::find_thread_path_by_id_str as find_conversation_path_by_id_str;
#[cfg(target_arch = "wasm32")]
pub use browser::first_rollout_content_match_snippet;
#[cfg(target_arch = "wasm32")]
pub use browser::get_threads;
#[cfg(target_arch = "wasm32")]
pub use browser::get_threads_in_root;
#[cfg(target_arch = "wasm32")]
pub use browser::open_rollout_line_reader;
#[cfg(target_arch = "wasm32")]
pub use browser::parse_cursor;
#[cfg(target_arch = "wasm32")]
pub use browser::plain_rollout_path;
#[cfg(target_arch = "wasm32")]
pub use browser::read_head_for_summary;
#[cfg(target_arch = "wasm32")]
pub use browser::read_session_meta_line;
#[cfg(target_arch = "wasm32")]
pub use browser::read_thread_item_from_rollout;
#[cfg(target_arch = "wasm32")]
pub use browser::remove_thread_name_entries;
#[cfg(target_arch = "wasm32")]
pub use browser::rollout_date_parts;
#[cfg(target_arch = "wasm32")]
pub use browser::search_rollout_matches;
#[cfg(target_arch = "wasm32")]
pub use browser::search_rollout_paths;
#[cfg(target_arch = "wasm32")]
pub use browser::spawn_rollout_compression_worker;
#[cfg(target_arch = "wasm32")]
pub use browser::sqlite_telemetry_recorder;
#[cfg(target_arch = "wasm32")]
pub use browser::state_db;
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
pub use config::Config;
pub use config::RolloutConfig;
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
pub use model_context::ModelContextScan;
pub use model_context::ModelContextScanProgress;
pub use persistence_metrics::RolloutPersistenceBatchMeasurement;
pub use persistence_metrics::RolloutPersistenceTelemetry;
pub use persistence_metrics::measure_and_filter_rollout_items;
pub use policy::is_persisted_rollout_item;
pub use policy::persisted_rollout_items;
pub use policy::should_persist_response_item_for_memories;
#[cfg(not(target_arch = "wasm32"))]
pub use recorder::RolloutRecorder;
#[cfg(not(target_arch = "wasm32"))]
pub use recorder::RolloutRecorderParams;
#[cfg(not(target_arch = "wasm32"))]
pub use recorder::append_rollout_item_to_path;
pub use reverse_jsonl_scanner::ReverseJsonlScanner;
pub use reverse_jsonl_scanner::ScanOutcome;
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

#[cfg(test)]
mod tests;
