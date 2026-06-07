use codex_app_server_protocol::ClientRequest;
use codex_app_server_protocol::JSONRPCErrorError;
use codex_app_server_protocol::Result as JsonRpcResult;
use codex_utils_absolute_path::AbsolutePathBuf;
use serde::de::DeserializeOwned;
use std::error::Error;
use std::fmt;
use std::io;

pub type RequestResult = std::result::Result<JsonRpcResult, JSONRPCErrorError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoteAppServerEndpoint {
    WebSocket {
        websocket_url: String,
        auth_token: Option<String>,
    },
    UnixSocket {
        socket_path: AbsolutePathBuf,
    },
}

#[derive(Debug, Clone)]
pub struct InProcessAppServerRequestHandle;

#[derive(Debug, Clone)]
pub struct RemoteAppServerRequestHandle;

#[derive(Debug, Clone)]
pub enum AppServerRequestHandle {
    InProcess(InProcessAppServerRequestHandle),
    Remote(RemoteAppServerRequestHandle),
}

#[derive(Debug)]
pub enum TypedRequestError {
    Transport {
        method: String,
        source: io::Error,
    },
    Server {
        method: String,
        source: JSONRPCErrorError,
    },
    Deserialize {
        method: String,
        source: serde_json::Error,
    },
}

impl fmt::Display for TypedRequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transport { method, source } => {
                write!(f, "{method} transport error: {source}")
            }
            Self::Server { method, source } => {
                write!(
                    f,
                    "{method} failed: {} (code {})",
                    source.message, source.code
                )
            }
            Self::Deserialize { method, source } => {
                write!(f, "{method} response decode error: {source}")
            }
        }
    }
}

impl Error for TypedRequestError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Transport { source, .. } => Some(source),
            Self::Server { .. } => None,
            Self::Deserialize { source, .. } => Some(source),
        }
    }
}

impl AppServerRequestHandle {
    pub async fn request(&self, request: ClientRequest) -> io::Result<RequestResult> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!(
                "{} is not wired to the browser app-server bridge yet",
                request_method_name(&request)
            ),
        ))
    }

    pub async fn request_typed<T>(&self, request: ClientRequest) -> Result<T, TypedRequestError>
    where
        T: DeserializeOwned,
    {
        let method = request_method_name(&request);
        let source = io::Error::new(
            io::ErrorKind::Unsupported,
            "browser app-server request bridge is not wired yet",
        );
        Err(TypedRequestError::Transport { method, source })
    }
}

fn request_method_name(request: &ClientRequest) -> String {
    match serde_json::to_value(request) {
        Ok(value) => value
            .get("method")
            .and_then(|method| method.as_str())
            .unwrap_or("unknown")
            .to_string(),
        Err(_) => "unknown".to_string(),
    }
}
