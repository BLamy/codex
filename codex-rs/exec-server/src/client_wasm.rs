use std::collections::VecDeque;

/// Placeholder for the JSON-RPC exec-server client on wasm.
///
/// Browser Codex installs host-backed process, filesystem, and HTTP
/// implementations directly. It does not open native WebSocket or stdio
/// exec-server transports from Rust.
#[derive(Clone, Debug)]
pub struct ExecServerClient;

#[derive(Debug, thiserror::Error)]
pub enum ExecServerError {
    #[error("failed to spawn exec-server: {0}")]
    Spawn(#[source] std::io::Error),
    #[error("exec-server transport closed")]
    Closed,
    #[error("{0}")]
    Disconnected(String),
    #[error("failed to serialize or deserialize exec-server JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("HTTP request failed: {0}")]
    HttpRequest(String),
    #[error("exec-server protocol error: {0}")]
    Protocol(String),
    #[error("exec-server rejected request ({code}): {message}")]
    Server { code: i64, message: String },
    #[error("environment registry configuration error: {0}")]
    EnvironmentRegistryConfig(String),
    #[error("environment registry authentication error: {0}")]
    EnvironmentRegistryAuth(String),
    #[error("environment registry request failed: {0}")]
    EnvironmentRegistryRequest(String),
}

/// Pull-based response body used by the browser host HTTP adapter.
pub struct HttpResponseBodyStream {
    chunks: VecDeque<Vec<u8>>,
}

impl HttpResponseBodyStream {
    pub(crate) fn buffered(body: Vec<u8>) -> Self {
        let mut chunks = VecDeque::new();
        if !body.is_empty() {
            chunks.push_back(body);
        }
        Self { chunks }
    }

    pub async fn recv(&mut self) -> Result<Option<Vec<u8>>, ExecServerError> {
        Ok(self.chunks.pop_front())
    }
}
