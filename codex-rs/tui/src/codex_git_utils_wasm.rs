use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitLogEntry {
    pub sha: String,
    pub timestamp: String,
    pub subject: String,
}

pub fn get_git_repo_root(_base_dir: &Path) -> Option<PathBuf> {
    None
}

pub async fn recent_commits(
    _cwd: &Path,
    _limit: usize,
) -> Vec<CommitLogEntry> {
    Vec::new()
}

pub async fn local_git_branches(_cwd: &Path) -> Vec<String> {
    Vec::new()
}

pub async fn current_branch_name(_cwd: &Path) -> Option<String> {
    None
}
