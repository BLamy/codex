use crate::store::PluginInstallResult;
use crate::store::validate_plugin_version_segment;
use codex_plugin::PluginId;
use codex_utils_absolute_path::AbsolutePathBuf;
use serde_json::Value as JsonValue;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ValidatedRemotePluginBundle {
    pub plugin_id: PluginId,
    pub plugin_version: String,
    pub app_manifest: Option<JsonValue>,
    pub bundle_download_url: String,
}

#[derive(Debug, thiserror::Error)]
pub enum RemotePluginBundleInstallError {
    #[error("{0}")]
    InvalidBundle(String),

    #[error(
        "browser remote plugin bundle installation requires an almostnode storage/archive host shim"
    )]
    BrowserHostShimRequired,
}

pub fn validate_remote_plugin_bundle(
    remote_plugin_id: &str,
    remote_marketplace_name: &str,
    plugin_name: &str,
    release_version: Option<&str>,
    bundle_download_url: Option<&str>,
    app_manifest: Option<JsonValue>,
) -> Result<ValidatedRemotePluginBundle, RemotePluginBundleInstallError> {
    let plugin_id = PluginId::new(plugin_name.to_string(), remote_marketplace_name.to_string())
        .map_err(|source| {
            RemotePluginBundleInstallError::InvalidBundle(format!(
                "backend returned an invalid local plugin id for remote plugin `{remote_plugin_id}`: {source}"
            ))
        })?;
    let plugin_version = release_version
        .map(str::trim)
        .filter(|version| !version.is_empty())
        .ok_or_else(|| {
            RemotePluginBundleInstallError::InvalidBundle(format!(
                "backend did not return a release version for remote plugin `{remote_plugin_id}`"
            ))
        })?
        .to_string();
    validate_plugin_version_segment(&plugin_version).map_err(|message| {
        RemotePluginBundleInstallError::InvalidBundle(format!(
            "backend returned an invalid release version for remote plugin `{remote_plugin_id}`: {message}"
        ))
    })?;
    let bundle_download_url = bundle_download_url
        .map(str::trim)
        .filter(|url| !url.is_empty())
        .ok_or_else(|| {
            RemotePluginBundleInstallError::InvalidBundle(format!(
                "backend did not return a download URL for remote plugin `{remote_plugin_id}`"
            ))
        })?
        .to_string();

    Ok(ValidatedRemotePluginBundle {
        plugin_id,
        plugin_version,
        app_manifest,
        bundle_download_url,
    })
}

pub async fn download_and_install_remote_plugin_bundle(
    _codex_home: PathBuf,
    _bundle: ValidatedRemotePluginBundle,
) -> Result<PluginInstallResult, RemotePluginBundleInstallError> {
    Err(RemotePluginBundleInstallError::BrowserHostShimRequired)
}

pub(crate) async fn download_and_extract_remote_plugin_bundle_to_path(
    _bundle: ValidatedRemotePluginBundle,
    _destination: AbsolutePathBuf,
) -> Result<AbsolutePathBuf, RemotePluginBundleInstallError> {
    Err(RemotePluginBundleInstallError::BrowserHostShimRequired)
}
