use codex_app_server_protocol::AppInfo;
use codex_file_search::FileMatch;
use codex_protocol::ThreadId;

#[derive(Clone, Debug)]
pub(crate) struct ConnectorsSnapshot {
    pub(crate) connectors: Vec<AppInfo>,
}

pub(crate) enum AppEvent {
    InsertHistoryCell(Box<dyn crate::history_cell::HistoryCell>),
    LookupMessageHistoryEntry {
        thread_id: ThreadId,
        offset: usize,
        log_id: u64,
    },
    StartFileSearch(String),
    FileSearchResult {
        query: String,
        matches: Vec<FileMatch>,
    },
}
