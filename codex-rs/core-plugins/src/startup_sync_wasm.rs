use std::path::Path;
use std::path::PathBuf;

const CURATED_PLUGINS_RELATIVE_DIR: &str = ".tmp/plugins";

pub fn curated_plugins_repo_path(codex_home: &Path) -> PathBuf {
    codex_home.join(CURATED_PLUGINS_RELATIVE_DIR)
}

pub fn read_curated_plugins_sha(_codex_home: &Path) -> Option<String> {
    None
}

pub fn sync_openai_plugins_repo(_codex_home: &Path) -> Result<String, String> {
    Err("browser curated plugin sync requires an almostnode git/archive host shim".to_string())
}

pub fn has_local_curated_plugins_snapshot(_codex_home: &Path) -> bool {
    false
}
