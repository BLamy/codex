use std::path::PathBuf;
use std::sync::Arc;

use crate::manager::PluginsConfigInput;
use crate::manager::PluginsManager;
use codex_login::AuthManager;

pub(crate) fn start_startup_remote_plugin_sync_once(
    _manager: Arc<PluginsManager>,
    _codex_home: PathBuf,
    _config: PluginsConfigInput,
    _auth_manager: Arc<AuthManager>,
) {
}
