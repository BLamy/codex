#[cfg(not(target_arch = "wasm32"))]
mod apply;
#[cfg(not(target_arch = "wasm32"))]
mod baseline;
#[cfg(not(target_arch = "wasm32"))]
mod branch;
#[cfg(not(target_arch = "wasm32"))]
mod errors;
mod fsmonitor;
#[cfg(not(target_arch = "wasm32"))]
mod info;
#[cfg(not(target_arch = "wasm32"))]
mod operations;
#[cfg(not(target_arch = "wasm32"))]
mod platform;
#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg(not(target_arch = "wasm32"))]
pub use apply::ApplyGitRequest;
#[cfg(not(target_arch = "wasm32"))]
pub use apply::ApplyGitResult;
#[cfg(not(target_arch = "wasm32"))]
pub use apply::apply_git_patch;
#[cfg(not(target_arch = "wasm32"))]
pub use apply::extract_paths_from_patch;
#[cfg(not(target_arch = "wasm32"))]
pub use apply::parse_git_apply_output;
#[cfg(not(target_arch = "wasm32"))]
pub use apply::stage_paths;
#[cfg(not(target_arch = "wasm32"))]
pub use baseline::GitBaselineChange;
#[cfg(not(target_arch = "wasm32"))]
pub use baseline::GitBaselineChangeStatus;
#[cfg(not(target_arch = "wasm32"))]
pub use baseline::GitBaselineDiff;
#[cfg(not(target_arch = "wasm32"))]
pub use baseline::diff_since_latest_init;
#[cfg(not(target_arch = "wasm32"))]
pub use baseline::ensure_git_baseline_repository;
#[cfg(not(target_arch = "wasm32"))]
pub use baseline::reset_git_repository;
#[cfg(not(target_arch = "wasm32"))]
pub use branch::merge_base_with_head;
#[cfg(not(target_arch = "wasm32"))]
pub use codex_protocol::protocol::GitSha;
#[cfg(not(target_arch = "wasm32"))]
pub use errors::GitToolingError;
pub use fsmonitor::FsmonitorOverride;
pub use fsmonitor::FsmonitorProbeRunner;
pub use fsmonitor::detect_fsmonitor_override;
#[cfg(not(target_arch = "wasm32"))]
pub use info::CommitLogEntry;
#[cfg(not(target_arch = "wasm32"))]
pub use info::GitDiffToRemote;
#[cfg(not(target_arch = "wasm32"))]
pub use info::GitInfo;
#[cfg(not(target_arch = "wasm32"))]
pub use info::canonicalize_git_remote_url;
#[cfg(not(target_arch = "wasm32"))]
pub use info::collect_git_info;
#[cfg(not(target_arch = "wasm32"))]
pub use info::current_branch_name;
#[cfg(not(target_arch = "wasm32"))]
pub use info::default_branch_name;
#[cfg(not(target_arch = "wasm32"))]
pub use info::get_git_remote_urls;
#[cfg(not(target_arch = "wasm32"))]
pub use info::get_git_remote_urls_assume_git_repo;
#[cfg(not(target_arch = "wasm32"))]
pub use info::get_git_repo_root;
#[cfg(not(target_arch = "wasm32"))]
pub use info::get_git_repo_root_with_fs;
#[cfg(not(target_arch = "wasm32"))]
pub use info::get_has_changes;
#[cfg(not(target_arch = "wasm32"))]
pub use info::get_head_commit_hash;
#[cfg(not(target_arch = "wasm32"))]
pub use info::git_diff_to_remote;
#[cfg(not(target_arch = "wasm32"))]
pub use info::local_git_branches;
#[cfg(not(target_arch = "wasm32"))]
pub use info::recent_commits;
#[cfg(not(target_arch = "wasm32"))]
pub use info::resolve_root_git_project_for_trust;
#[cfg(not(target_arch = "wasm32"))]
pub use platform::create_symlink;

#[cfg(target_arch = "wasm32")]
pub use wasm::*;
