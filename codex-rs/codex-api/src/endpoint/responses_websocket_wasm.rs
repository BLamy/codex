use crate::auth::SharedAuthProvider;
use crate::common::ResponseStream;
use crate::common::ResponsesWsRequest;
use crate::error::ApiError;
use crate::provider::Provider;
use crate::telemetry::WebsocketTelemetry;
use codex_http_client::HttpClientFactory;
use http::HeaderMap;
use http::StatusCode;
use std::sync::Arc;
use std::sync::OnceLock;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ResponsesWebsocketConnection;

impl ResponsesWebsocketConnection {
    pub async fn is_closed(&self) -> bool {
        true
    }

    pub async fn stream_request(
        &self,
        _request: ResponsesWsRequest<'_>,
        _connection_reused: bool,
        _turn_state: Option<Arc<OnceLock<String>>>,
    ) -> Result<ResponseStream, ApiError> {
        Err(unsupported())
    }
}

/// Client for connecting to the Responses WebSocket endpoint for one provider.
pub struct ResponsesWebsocketClient {
    _provider: Provider,
    _auth: SharedAuthProvider,
}

/// Close frame information captured by a handshake probe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResponsesWebsocketClose {
    /// WebSocket close code returned by the server.
    pub code: String,
    /// Human-readable close reason returned by the server.
    pub reason: String,
}

/// Result of a handshake-only Responses WebSocket probe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResponsesWebsocketProbe {
    /// Redacted by callers before displaying or serializing support reports.
    pub url: String,
    /// HTTP status returned by the successful WebSocket upgrade.
    pub status: StatusCode,
    /// Whether the server reported reasoning support in the upgrade response.
    pub reasoning_included: bool,
    /// Whether the server returned a model catalog ETag in the upgrade response.
    pub models_etag_present: bool,
    /// Whether the server returned a server-selected model in the upgrade response.
    pub server_model_present: bool,
    /// Close frame received immediately after upgrade, when one arrives quickly.
    pub immediate_close: Option<ResponsesWebsocketClose>,
}

impl ResponsesWebsocketClient {
    /// Creates a Responses WebSocket client for an already-resolved provider and auth source.
    pub fn new(provider: Provider, auth: SharedAuthProvider) -> Self {
        Self {
            _provider: provider,
            _auth: auth,
        }
    }

    pub async fn connect(
        &self,
        _http_client_factory: &HttpClientFactory,
        _extra_headers: HeaderMap,
        _default_headers: HeaderMap,
        _turn_state: Option<Arc<OnceLock<String>>>,
        _telemetry: Option<Arc<dyn WebsocketTelemetry>>,
    ) -> Result<ResponsesWebsocketConnection, ApiError> {
        Err(unsupported())
    }

    pub async fn probe_handshake(
        &self,
        _http_client_factory: &HttpClientFactory,
        _extra_headers: HeaderMap,
        _default_headers: HeaderMap,
        _immediate_close_timeout: Duration,
    ) -> Result<ResponsesWebsocketProbe, ApiError> {
        Err(unsupported())
    }
}

fn unsupported() -> ApiError {
    ApiError::Stream(
        "responses websocket transport is not available on wasm32; use HTTP streaming".to_string(),
    )
}
