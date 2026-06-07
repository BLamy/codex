use std::collections::BTreeMap;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitStatus;
use std::string::FromUtf8Error;

use codex_file_system::ExecutorFileSystem;
pub use codex_protocol::protocol::GitSha;
use codex_utils_absolute_path::AbsolutePathBuf;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;
use ts_rs::TS;
use walkdir::Error as WalkdirError;

const UNSUPPORTED_GIT: &str = "browser git operations require an almostnode git/process host shim";

#[derive(Debug, Clone)]
pub struct ApplyGitRequest {
    pub cwd: PathBuf,
    pub diff: String,
    pub revert: bool,
    pub preflight: bool,
}

#[derive(Debug, Clone)]
pub struct ApplyGitResult {
    pub exit_code: i32,
    pub applied_paths: Vec<String>,
    pub skipped_paths: Vec<String>,
    pub conflicted_paths: Vec<String>,
    pub stdout: String,
    pub stderr: String,
    pub cmd_for_log: String,
}

pub fn apply_git_patch(_req: &ApplyGitRequest) -> io::Result<ApplyGitResult> {
    Err(io::Error::other(UNSUPPORTED_GIT))
}

pub fn stage_paths(_git_root: &Path, _diff: &str) -> io::Result<()> {
    Err(io::Error::other(UNSUPPORTED_GIT))
}

pub fn extract_paths_from_patch(diff_text: &str) -> Vec<String> {
    let mut set = std::collections::BTreeSet::new();
    for raw_line in diff_text.lines() {
        let line = raw_line.trim();
        let Some(rest) = line.strip_prefix("diff --git ") else {
            continue;
        };
        let mut parts = rest.split_whitespace();
        if let Some(path) = parts
            .next()
            .and_then(|path| normalize_diff_path(path, "a/"))
        {
            set.insert(path);
        }
        if let Some(path) = parts
            .next()
            .and_then(|path| normalize_diff_path(path, "b/"))
        {
            set.insert(path);
        }
    }
    set.into_iter().collect()
}

fn normalize_diff_path(raw: &str, prefix: &str) -> Option<String> {
    let trimmed = raw.trim().trim_matches('"');
    trimmed
        .strip_prefix(prefix)
        .filter(|path| !path.is_empty() && *path != "/dev/null")
        .map(str::to_string)
}

pub fn parse_git_apply_output(
    _stdout: &str,
    _stderr: &str,
) -> (Vec<String>, Vec<String>, Vec<String>) {
    (Vec::new(), Vec::new(), Vec::new())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitBaselineChangeStatus {
    Added,
    Modified,
    Deleted,
}

impl GitBaselineChangeStatus {
    pub fn label(self) -> &'static str {
        match self {
            GitBaselineChangeStatus::Added => "A",
            GitBaselineChangeStatus::Modified => "M",
            GitBaselineChangeStatus::Deleted => "D",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitBaselineChange {
    pub status: GitBaselineChangeStatus,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitBaselineDiff {
    pub changes: Vec<GitBaselineChange>,
    pub unified_diff: String,
}

impl GitBaselineDiff {
    pub fn has_changes(&self) -> bool {
        !self.changes.is_empty()
    }
}

pub async fn reset_git_repository(_root: &Path) -> anyhow::Result<()> {
    Err(anyhow::anyhow!(UNSUPPORTED_GIT))
}

pub async fn ensure_git_baseline_repository(_root: &Path) -> anyhow::Result<()> {
    Err(anyhow::anyhow!(UNSUPPORTED_GIT))
}

pub async fn diff_since_latest_init(_root: &Path) -> anyhow::Result<GitBaselineDiff> {
    Err(anyhow::anyhow!(UNSUPPORTED_GIT))
}

pub fn merge_base_with_head(
    _repo_path: &Path,
    _branch: &str,
) -> Result<Option<String>, GitToolingError> {
    Err(GitToolingError::UnsupportedInBrowser)
}

#[derive(Debug, Error)]
pub enum GitToolingError {
    #[error("git command `{command}` failed with status {status}: {stderr}")]
    GitCommand {
        command: String,
        status: ExitStatus,
        stderr: String,
    },
    #[error("git command `{command}` produced non-UTF-8 output")]
    GitOutputUtf8 {
        command: String,
        #[source]
        source: FromUtf8Error,
    },
    #[error("{path:?} is not a git repository")]
    NotAGitRepository { path: PathBuf },
    #[error("path {path:?} must be relative to the repository root")]
    NonRelativePath { path: PathBuf },
    #[error("path {path:?} escapes the repository root")]
    PathEscapesRepository { path: PathBuf },
    #[error("failed to process path inside worktree")]
    PathPrefix(#[from] std::path::StripPrefixError),
    #[error(transparent)]
    Walkdir(#[from] WalkdirError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("{UNSUPPORTED_GIT}")]
    UnsupportedInBrowser,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, TS)]
pub struct GitInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_hash: Option<GitSha>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository_url: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GitDiffToRemote {
    pub sha: GitSha,
    pub diff: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommitLogEntry {
    pub sha: String,
    pub timestamp: i64,
    pub subject: String,
}

pub fn get_git_repo_root(base_dir: &Path) -> Option<PathBuf> {
    let mut current = if base_dir.is_dir() {
        base_dir.to_path_buf()
    } else {
        base_dir.parent()?.to_path_buf()
    };

    loop {
        if current.join(".git").exists() {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

pub async fn get_git_repo_root_with_fs(
    _fs: &dyn ExecutorFileSystem,
    _cwd: &AbsolutePathBuf,
) -> Option<AbsolutePathBuf> {
    None
}

pub async fn collect_git_info(_cwd: &Path) -> Option<GitInfo> {
    None
}

pub async fn get_git_remote_urls(_cwd: &Path) -> Option<BTreeMap<String, String>> {
    None
}

pub async fn get_git_remote_urls_assume_git_repo(_cwd: &Path) -> Option<BTreeMap<String, String>> {
    None
}

pub async fn get_head_commit_hash(_cwd: &Path) -> Option<GitSha> {
    None
}

pub fn canonicalize_git_remote_url(_url: &str) -> Option<String> {
    None
}

pub async fn get_has_changes(_cwd: &Path) -> Option<bool> {
    None
}

pub async fn recent_commits(_cwd: &Path, _limit: usize) -> Vec<CommitLogEntry> {
    Vec::new()
}

pub async fn git_diff_to_remote(_cwd: &Path) -> Option<GitDiffToRemote> {
    None
}

pub async fn default_branch_name(_cwd: &Path) -> Option<String> {
    None
}

pub async fn local_git_branches(_cwd: &Path) -> Vec<String> {
    Vec::new()
}

pub async fn current_branch_name(_cwd: &Path) -> Option<String> {
    None
}

pub async fn resolve_root_git_project_for_trust(
    _fs: &dyn ExecutorFileSystem,
    _cwd: &AbsolutePathBuf,
) -> Option<AbsolutePathBuf> {
    None
}

pub fn create_symlink(
    _source: &Path,
    _link_target: &Path,
    _destination: &Path,
) -> Result<(), GitToolingError> {
    Err(GitToolingError::UnsupportedInBrowser)
}
