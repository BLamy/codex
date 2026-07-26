//! Browser-safe boundary for the native RMCP client.
//!
//! Codex MCP currently relies on process transports, native HTTP/OAuth support,
//! and keyring-backed credentials that are not available in the browser build.
//! Keep the upstream public API intact so the rest of Codex can compile, while
//! reporting the missing host capability explicitly at runtime.

use std::collections::HashMap;
use std::ffi::OsString;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use anyhow::anyhow;
use codex_api::SharedAuthProvider;
use codex_config::types::AuthKeyringBackendKind;
use codex_config::types::McpServerEnvVar;
use codex_config::types::OAuthCredentialsStoreMode;
use codex_exec_server::ExecBackend;
use codex_exec_server::HttpClient;
use codex_protocol::protocol::McpAuthStatus;
use futures::future::BoxFuture;
use rmcp::model::CallToolResult;
use rmcp::model::CreateElicitationRequestParams;
use rmcp::model::CreateElicitationResult;
use rmcp::model::ElicitationAction;
use rmcp::model::InitializeRequestParams;
use rmcp::model::InitializeResult;
use rmcp::model::ListResourceTemplatesResult;
use rmcp::model::ListResourcesResult;
use rmcp::model::ListToolsResult;
use rmcp::model::PaginatedRequestParams;
use rmcp::model::ReadResourceRequestParams;
use rmcp::model::ReadResourceResult;
use rmcp::model::RequestId;
use rmcp::model::RequestParamsMeta;
use rmcp::model::ServerResult;
use rmcp::model::Tool;
use serde::Deserialize;
use serde::Serialize;
use tokio::sync::oneshot;

const UNSUPPORTED_MCP: &str =
    "browser MCP requires an Almost Node host transport before native Codex MCP can run";
const UNSUPPORTED_OAUTH: &str =
    "browser MCP OAuth requires an Almost Node credential and callback host";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamableHttpOAuthDiscovery {
    pub scopes_supported: Option<Vec<String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpLoginRequirement {
    Login,
    Reauthentication,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpAuthState {
    Unsupported,
    LoggedOut(McpLoginRequirement),
    BearerToken,
    OAuth,
}

impl From<McpAuthState> for McpAuthStatus {
    fn from(value: McpAuthState) -> Self {
        match value {
            McpAuthState::Unsupported => Self::Unsupported,
            McpAuthState::LoggedOut(_) => Self::NotLoggedIn,
            McpAuthState::BearerToken => Self::BearerToken,
            McpAuthState::OAuth => Self::OAuth,
        }
    }
}

fn has_authorization_header(headers: Option<&HashMap<String, String>>) -> bool {
    headers.is_some_and(|headers| {
        headers
            .keys()
            .any(|key| key.eq_ignore_ascii_case("authorization"))
    })
}

fn browser_auth_state(
    bearer_token_env_var: Option<&str>,
    http_headers: Option<&HashMap<String, String>>,
    env_http_headers: Option<&HashMap<String, String>>,
) -> McpAuthState {
    if bearer_token_env_var.is_some()
        || has_authorization_header(http_headers)
        || has_authorization_header(env_http_headers)
    {
        McpAuthState::BearerToken
    } else {
        McpAuthState::Unsupported
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn determine_streamable_http_auth_status(
    _server_name: &str,
    _url: &str,
    bearer_token_env_var: Option<&str>,
    http_headers: Option<HashMap<String, String>>,
    env_http_headers: Option<HashMap<String, String>>,
    _store_mode: OAuthCredentialsStoreMode,
    _keyring_backend_kind: AuthKeyringBackendKind,
) -> Result<McpAuthState> {
    Ok(browser_auth_state(
        bearer_token_env_var,
        http_headers.as_ref(),
        env_http_headers.as_ref(),
    ))
}

#[allow(clippy::too_many_arguments)]
pub async fn determine_streamable_http_auth_status_with_http_client(
    server_name: &str,
    url: &str,
    bearer_token_env_var: Option<&str>,
    http_headers: Option<HashMap<String, String>>,
    env_http_headers: Option<HashMap<String, String>>,
    store_mode: OAuthCredentialsStoreMode,
    keyring_backend_kind: AuthKeyringBackendKind,
    _http_client: Arc<dyn HttpClient>,
) -> Result<McpAuthState> {
    determine_streamable_http_auth_status(
        server_name,
        url,
        bearer_token_env_var,
        http_headers,
        env_http_headers,
        store_mode,
        keyring_backend_kind,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
pub fn determine_streamable_http_auth_status_from_credentials(
    _server_name: &str,
    _url: &str,
    bearer_token_env_var: Option<&str>,
    http_headers: Option<HashMap<String, String>>,
    env_http_headers: Option<HashMap<String, String>>,
    _store_mode: OAuthCredentialsStoreMode,
    _keyring_backend_kind: AuthKeyringBackendKind,
) -> Result<Option<McpAuthState>> {
    Ok(Some(browser_auth_state(
        bearer_token_env_var,
        http_headers.as_ref(),
        env_http_headers.as_ref(),
    )))
}

pub async fn supports_oauth_login(_url: &str) -> Result<bool> {
    Ok(false)
}

pub async fn discover_streamable_http_oauth(
    _url: &str,
    _http_headers: Option<HashMap<String, String>>,
    _env_http_headers: Option<HashMap<String, String>>,
) -> Result<Option<StreamableHttpOAuthDiscovery>> {
    Ok(None)
}

pub async fn discover_streamable_http_oauth_with_http_client(
    _url: &str,
    _http_headers: Option<HashMap<String, String>>,
    _env_http_headers: Option<HashMap<String, String>>,
    _http_client: Arc<dyn HttpClient>,
) -> Result<Option<StreamableHttpOAuthDiscovery>> {
    Ok(None)
}

pub fn is_authentication_required_error(_error: &anyhow::Error) -> bool {
    false
}

pub trait InProcessTransportFactory: Send + Sync {}

pub trait StdioServerLauncher: Send + Sync {}

#[derive(Clone)]
pub struct LocalStdioServerLauncher {
    _fallback_cwd: PathBuf,
}

impl LocalStdioServerLauncher {
    pub fn new(fallback_cwd: PathBuf) -> Self {
        Self {
            _fallback_cwd: fallback_cwd,
        }
    }
}

impl StdioServerLauncher for LocalStdioServerLauncher {}

#[derive(Clone)]
pub struct ExecutorStdioServerLauncher {
    _exec_backend: Arc<dyn ExecBackend>,
}

impl ExecutorStdioServerLauncher {
    pub fn new(exec_backend: Arc<dyn ExecBackend>) -> Self {
        Self {
            _exec_backend: exec_backend,
        }
    }
}

impl StdioServerLauncher for ExecutorStdioServerLauncher {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StoredOAuthTokens {
    pub server_name: String,
    pub url: String,
    pub client_id: String,
    pub token_response: WrappedOAuthTokenResponse,
    #[serde(default)]
    pub expires_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WrappedOAuthTokenResponse(pub serde_json::Value);

pub fn save_oauth_tokens(
    _server_name: &str,
    _tokens: &StoredOAuthTokens,
    _store_mode: OAuthCredentialsStoreMode,
    _keyring_backend_kind: AuthKeyringBackendKind,
) -> Result<()> {
    Err(anyhow!(UNSUPPORTED_OAUTH))
}

pub fn delete_oauth_tokens(
    _server_name: &str,
    _url: &str,
    _store_mode: OAuthCredentialsStoreMode,
    _keyring_backend_kind: AuthKeyringBackendKind,
) -> Result<bool> {
    Err(anyhow!(UNSUPPORTED_OAUTH))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OAuthProviderError {
    error: Option<String>,
    error_description: Option<String>,
}

impl OAuthProviderError {
    pub fn new(error: Option<String>, error_description: Option<String>) -> Self {
        Self {
            error,
            error_description,
        }
    }
}

impl std::fmt::Display for OAuthProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (self.error.as_deref(), self.error_description.as_deref()) {
            (Some(error), Some(error_description)) => {
                write!(f, "OAuth provider returned `{error}`: {error_description}")
            }
            (Some(error), None) => write!(f, "OAuth provider returned `{error}`"),
            (None, Some(error_description)) => write!(f, "OAuth error: {error_description}"),
            (None, None) => write!(f, "OAuth provider returned an error"),
        }
    }
}

impl std::error::Error for OAuthProviderError {}

#[allow(clippy::too_many_arguments)]
pub async fn perform_oauth_login(
    _server_name: &str,
    _server_url: &str,
    _store_mode: OAuthCredentialsStoreMode,
    _keyring_backend_kind: AuthKeyringBackendKind,
    _http_headers: Option<HashMap<String, String>>,
    _env_http_headers: Option<HashMap<String, String>>,
    _scopes: &[String],
    _oauth_client_id: Option<&str>,
    _oauth_resource: Option<&str>,
    _callback_port: Option<u16>,
    _callback_url: Option<&str>,
) -> Result<()> {
    Err(anyhow!(UNSUPPORTED_OAUTH))
}

#[allow(clippy::too_many_arguments)]
pub async fn perform_oauth_login_silent(
    server_name: &str,
    server_url: &str,
    store_mode: OAuthCredentialsStoreMode,
    keyring_backend_kind: AuthKeyringBackendKind,
    http_headers: Option<HashMap<String, String>>,
    env_http_headers: Option<HashMap<String, String>>,
    scopes: &[String],
    oauth_client_id: Option<&str>,
    oauth_resource: Option<&str>,
    callback_port: Option<u16>,
    callback_url: Option<&str>,
) -> Result<()> {
    perform_oauth_login(
        server_name,
        server_url,
        store_mode,
        keyring_backend_kind,
        http_headers,
        env_http_headers,
        scopes,
        oauth_client_id,
        oauth_resource,
        callback_port,
        callback_url,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn perform_oauth_login_return_url(
    _server_name: &str,
    _server_url: &str,
    _store_mode: OAuthCredentialsStoreMode,
    _keyring_backend_kind: AuthKeyringBackendKind,
    _http_headers: Option<HashMap<String, String>>,
    _env_http_headers: Option<HashMap<String, String>>,
    _scopes: &[String],
    _oauth_client_id: Option<&str>,
    _oauth_resource: Option<&str>,
    _timeout_secs: Option<i64>,
    _callback_port: Option<u16>,
    _callback_url: Option<&str>,
) -> Result<OauthLoginHandle> {
    Err(anyhow!(UNSUPPORTED_OAUTH))
}

#[allow(clippy::too_many_arguments)]
pub async fn perform_oauth_login_return_url_with_http_client(
    _server_name: &str,
    _server_url: &str,
    _store_mode: OAuthCredentialsStoreMode,
    _keyring_backend_kind: AuthKeyringBackendKind,
    _http_headers: Option<HashMap<String, String>>,
    _env_http_headers: Option<HashMap<String, String>>,
    _scopes: &[String],
    _oauth_client_id: Option<&str>,
    _oauth_resource: Option<&str>,
    _timeout_secs: Option<i64>,
    _callback_port: Option<u16>,
    _callback_url: Option<&str>,
    _http_client: Arc<dyn HttpClient>,
) -> Result<OauthLoginHandle> {
    Err(anyhow!(UNSUPPORTED_OAUTH))
}

pub struct OauthLoginHandle {
    authorization_url: String,
    completion: oneshot::Receiver<Result<()>>,
}

impl OauthLoginHandle {
    pub fn authorization_url(&self) -> &str {
        &self.authorization_url
    }

    pub fn into_parts(self) -> (String, oneshot::Receiver<Result<()>>) {
        (self.authorization_url, self.completion)
    }

    pub async fn wait(self) -> Result<()> {
        self.completion
            .await
            .map_err(|err| anyhow!("OAuth login task was cancelled: {err}"))?
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Elicitation {
    Mcp(CreateElicitationRequestParams),
    OpenAiForm {
        meta: Option<serde_json::Value>,
        message: String,
        requested_schema: serde_json::Value,
    },
}

impl Elicitation {
    pub fn meta(&self) -> Option<&serde_json::Map<String, serde_json::Value>> {
        match self {
            Self::Mcp(request) => request.meta().map(|meta| &meta.0),
            Self::OpenAiForm { meta, .. } => meta.as_ref().and_then(serde_json::Value::as_object),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ElicitationResponse {
    pub action: ElicitationAction,
    pub content: Option<serde_json::Value>,
    #[serde(rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

impl From<CreateElicitationResult> for ElicitationResponse {
    fn from(value: CreateElicitationResult) -> Self {
        Self {
            action: value.action,
            content: value.content,
            meta: None,
        }
    }
}

impl From<ElicitationResponse> for CreateElicitationResult {
    fn from(value: ElicitationResponse) -> Self {
        Self {
            action: value.action,
            content: value.content,
            meta: None,
        }
    }
}

pub type SendElicitation = Box<
    dyn Fn(RequestId, Elicitation) -> BoxFuture<'static, Result<ElicitationResponse>> + Send + Sync,
>;

pub struct ToolWithConnectorId {
    pub tool: Tool,
    pub connector_id: Option<String>,
    pub connector_name: Option<String>,
    pub connector_description: Option<String>,
}

pub struct ListToolsWithConnectorIdResult {
    pub next_cursor: Option<String>,
    pub tools: Vec<ToolWithConnectorId>,
}

pub struct RmcpClient;

impl RmcpClient {
    pub async fn new_in_process_client(
        _factory: Arc<dyn InProcessTransportFactory>,
    ) -> io::Result<Self> {
        Err(io::Error::other(UNSUPPORTED_MCP))
    }

    pub async fn new_stdio_client(
        _program: OsString,
        _args: Vec<OsString>,
        _env: Option<HashMap<OsString, OsString>>,
        _env_vars: &[McpServerEnvVar],
        _cwd: Option<String>,
        _launcher: Arc<dyn StdioServerLauncher>,
    ) -> io::Result<Self> {
        Err(io::Error::other(UNSUPPORTED_MCP))
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn new_streamable_http_client(
        _server_name: &str,
        _url: &str,
        _bearer_token: Option<String>,
        _http_headers: Option<HashMap<String, String>>,
        _env_http_headers: Option<HashMap<String, String>>,
        _store_mode: OAuthCredentialsStoreMode,
        _keyring_backend_kind: AuthKeyringBackendKind,
        _http_client: Arc<dyn HttpClient>,
        _auth_provider: Option<SharedAuthProvider>,
    ) -> Result<Self> {
        Err(anyhow!(UNSUPPORTED_MCP))
    }

    pub async fn initialize(
        &self,
        _params: InitializeRequestParams,
        _timeout: Option<Duration>,
        _send_elicitation: SendElicitation,
    ) -> Result<InitializeResult> {
        Err(anyhow!(UNSUPPORTED_MCP))
    }

    pub async fn list_tools(
        &self,
        _params: Option<PaginatedRequestParams>,
        _timeout: Option<Duration>,
    ) -> Result<ListToolsResult> {
        Err(anyhow!(UNSUPPORTED_MCP))
    }

    pub async fn list_tools_with_connector_ids(
        &self,
        _params: Option<PaginatedRequestParams>,
        _timeout: Option<Duration>,
    ) -> Result<ListToolsWithConnectorIdResult> {
        Err(anyhow!(UNSUPPORTED_MCP))
    }

    pub async fn list_resources(
        &self,
        _params: Option<PaginatedRequestParams>,
        _timeout: Option<Duration>,
    ) -> Result<ListResourcesResult> {
        Err(anyhow!(UNSUPPORTED_MCP))
    }

    pub async fn list_resource_templates(
        &self,
        _params: Option<PaginatedRequestParams>,
        _timeout: Option<Duration>,
    ) -> Result<ListResourceTemplatesResult> {
        Err(anyhow!(UNSUPPORTED_MCP))
    }

    pub async fn read_resource(
        &self,
        _params: ReadResourceRequestParams,
        _timeout: Option<Duration>,
    ) -> Result<ReadResourceResult> {
        Err(anyhow!(UNSUPPORTED_MCP))
    }

    pub async fn call_tool(
        &self,
        _name: String,
        _arguments: Option<serde_json::Value>,
        _meta: Option<serde_json::Value>,
        _timeout: Option<Duration>,
    ) -> Result<CallToolResult> {
        Err(anyhow!(UNSUPPORTED_MCP))
    }

    pub async fn send_custom_notification(
        &self,
        _method: &str,
        _params: Option<serde_json::Value>,
    ) -> Result<()> {
        Err(anyhow!(UNSUPPORTED_MCP))
    }

    pub async fn send_custom_request(
        &self,
        _method: &str,
        _params: Option<serde_json::Value>,
    ) -> Result<ServerResult> {
        Err(anyhow!(UNSUPPORTED_MCP))
    }

    pub async fn shutdown(&self) {}
}
