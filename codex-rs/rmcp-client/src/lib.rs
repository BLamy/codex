#[cfg(not(target_arch = "wasm32"))]
mod auth_status;
#[cfg(not(target_arch = "wasm32"))]
mod elicitation_client_service;
#[cfg(not(target_arch = "wasm32"))]
mod executor_process_transport;
#[cfg(not(target_arch = "wasm32"))]
mod http_client_adapter;
#[cfg(not(target_arch = "wasm32"))]
mod in_process_transport;
#[cfg(not(target_arch = "wasm32"))]
mod logging_client_handler;
#[cfg(not(target_arch = "wasm32"))]
mod oauth;
#[cfg(not(target_arch = "wasm32"))]
mod perform_oauth_login;
#[cfg(not(target_arch = "wasm32"))]
mod program_resolver;
#[cfg(not(target_arch = "wasm32"))]
mod rmcp_client;
#[cfg(not(target_arch = "wasm32"))]
mod stdio_server_launcher;
#[cfg(not(target_arch = "wasm32"))]
mod utils;
#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg(not(target_arch = "wasm32"))]
pub use auth_status::StreamableHttpOAuthDiscovery;
#[cfg(not(target_arch = "wasm32"))]
pub use auth_status::determine_streamable_http_auth_status;
#[cfg(not(target_arch = "wasm32"))]
pub use auth_status::discover_streamable_http_oauth;
#[cfg(not(target_arch = "wasm32"))]
pub use auth_status::supports_oauth_login;
#[cfg(not(target_arch = "wasm32"))]
pub use codex_protocol::protocol::McpAuthStatus;
#[cfg(not(target_arch = "wasm32"))]
pub use in_process_transport::InProcessTransportFactory;
#[cfg(not(target_arch = "wasm32"))]
pub use oauth::StoredOAuthTokens;
#[cfg(not(target_arch = "wasm32"))]
pub use oauth::WrappedOAuthTokenResponse;
#[cfg(not(target_arch = "wasm32"))]
pub use oauth::delete_oauth_tokens;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use oauth::load_oauth_tokens;
#[cfg(not(target_arch = "wasm32"))]
pub use oauth::save_oauth_tokens;
#[cfg(not(target_arch = "wasm32"))]
pub use perform_oauth_login::OAuthProviderError;
#[cfg(not(target_arch = "wasm32"))]
pub use perform_oauth_login::OauthLoginHandle;
#[cfg(not(target_arch = "wasm32"))]
pub use perform_oauth_login::perform_oauth_login;
#[cfg(not(target_arch = "wasm32"))]
pub use perform_oauth_login::perform_oauth_login_return_url;
#[cfg(not(target_arch = "wasm32"))]
pub use perform_oauth_login::perform_oauth_login_silent;
#[cfg(not(target_arch = "wasm32"))]
pub use rmcp::model::ElicitationAction;
#[cfg(not(target_arch = "wasm32"))]
pub use rmcp_client::Elicitation;
#[cfg(not(target_arch = "wasm32"))]
pub use rmcp_client::ElicitationResponse;
#[cfg(not(target_arch = "wasm32"))]
pub use rmcp_client::ListToolsWithConnectorIdResult;
#[cfg(not(target_arch = "wasm32"))]
pub use rmcp_client::RmcpClient;
#[cfg(not(target_arch = "wasm32"))]
pub use rmcp_client::SendElicitation;
#[cfg(not(target_arch = "wasm32"))]
pub use rmcp_client::ToolWithConnectorId;
#[cfg(not(target_arch = "wasm32"))]
pub use stdio_server_launcher::ExecutorStdioServerLauncher;
#[cfg(not(target_arch = "wasm32"))]
pub use stdio_server_launcher::LocalStdioServerLauncher;
#[cfg(not(target_arch = "wasm32"))]
pub use stdio_server_launcher::StdioServerLauncher;

#[cfg(target_arch = "wasm32")]
pub use wasm::*;
