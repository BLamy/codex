use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchType {
    File,
    Directory,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileMatch {
    pub score: u32,
    pub path: PathBuf,
    pub match_type: MatchType,
    pub root: PathBuf,
    pub indices: Option<Vec<u32>>,
}

impl FileMatch {
    pub fn full_path(&self) -> PathBuf {
        self.root.join(&self.path)
    }
}

#[derive(Debug, Clone)]
pub struct FileSearchOptions {
    pub compute_indices: bool,
}

impl Default for FileSearchOptions {
    fn default() -> Self {
        Self {
            compute_indices: false,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FileSearchSnapshot {
    pub query: String,
    pub matches: Vec<FileMatch>,
}

pub trait SessionReporter: Send + Sync {
    fn on_update(&self, snapshot: &FileSearchSnapshot);
    fn on_complete(&self);
}

pub struct FileSearchSession {
    _roots: Vec<PathBuf>,
    _options: FileSearchOptions,
    _reporter: Arc<dyn SessionReporter>,
}

impl FileSearchSession {
    pub fn update_query(&self, _query: &str) {}
}

pub fn create_session(
    roots: Vec<PathBuf>,
    options: FileSearchOptions,
    reporter: Arc<dyn SessionReporter>,
    _cancel_flag: Option<Arc<std::sync::atomic::AtomicBool>>,
) -> std::io::Result<FileSearchSession> {
    let _ = options.compute_indices;
    Ok(FileSearchSession {
        _roots: roots,
        _options: options,
        _reporter: reporter,
    })
}
