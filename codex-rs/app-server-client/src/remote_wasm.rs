use std::io::Error as IoError;
use std::io::ErrorKind;
use std::io::Result as IoResult;

use codex_app_server_protocol::ClientNotification;
use codex_app_server_protocol::ClientRequest;
use codex_app_server_protocol::InitializeCapabilities;
use codex_app_server_protocol::JSONRPCErrorError;
use codex_app_server_protocol::RequestId;
use codex_app_server_protocol::Result as JsonRpcResult;
use codex_utils_absolute_path::AbsolutePathBuf;
use serde::de::DeserializeOwned;

use crate::AppServerEvent;
use crate::RequestResult;
use crate::TypedRequestError;
use crate::request_method_name;

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
pub struct RemoteAppServerConnectArgs {
    pub endpoint: RemoteAppServerEndpoint,
    pub client_name: String,
    pub client_version: String,
    pub experimental_api: bool,
    pub opt_out_notification_methods: Vec<String>,
    pub channel_capacity: usize,
}

pub struct RemoteAppServerClient;

#[derive(Clone)]
pub struct RemoteAppServerRequestHandle;

fn unsupported_remote_transport() -> IoError {
    IoError::new(
        ErrorKind::Unsupported,
        "remote app-server websocket and Unix socket transports are native-only on wasm32",
    )
}

impl RemoteAppServerConnectArgs {
    pub(crate) fn initialize_params(&self) -> codex_app_server_protocol::InitializeParams {
        let capabilities = InitializeCapabilities {
            experimental_api: self.experimental_api,
            request_attestation: false,
            opt_out_notification_methods: if self.opt_out_notification_methods.is_empty() {
                None
            } else {
                Some(self.opt_out_notification_methods.clone())
            },
        };

        codex_app_server_protocol::InitializeParams {
            client_info: codex_app_server_protocol::ClientInfo {
                name: self.client_name.clone(),
                title: None,
                version: self.client_version.clone(),
            },
            capabilities: Some(capabilities),
        }
    }
}

impl RemoteAppServerClient {
    pub async fn connect(_args: RemoteAppServerConnectArgs) -> IoResult<Self> {
        Err(unsupported_remote_transport())
    }

    pub fn server_version(&self) -> Option<&str> {
        None
    }

    pub fn request_handle(&self) -> RemoteAppServerRequestHandle {
        RemoteAppServerRequestHandle
    }

    pub async fn request(&self, _request: ClientRequest) -> IoResult<RequestResult> {
        Err(unsupported_remote_transport())
    }

    pub async fn request_typed<T>(&self, request: ClientRequest) -> Result<T, TypedRequestError>
    where
        T: DeserializeOwned,
    {
        let method = request_method_name(&request);
        Err(TypedRequestError::Transport {
            method,
            source: unsupported_remote_transport(),
        })
    }

    pub async fn notify(&self, _notification: ClientNotification) -> IoResult<()> {
        Err(unsupported_remote_transport())
    }

    pub async fn resolve_server_request(
        &self,
        _request_id: RequestId,
        _result: JsonRpcResult,
    ) -> IoResult<()> {
        Err(unsupported_remote_transport())
    }

    pub async fn reject_server_request(
        &self,
        _request_id: RequestId,
        _error: JSONRPCErrorError,
    ) -> IoResult<()> {
        Err(unsupported_remote_transport())
    }

    pub async fn next_event(&mut self) -> Option<AppServerEvent> {
        None
    }

    pub async fn shutdown(self) -> IoResult<()> {
        Ok(())
    }
}

impl RemoteAppServerRequestHandle {
    pub async fn request(&self, _request: ClientRequest) -> IoResult<RequestResult> {
        Err(unsupported_remote_transport())
    }

    pub async fn request_typed<T>(&self, request: ClientRequest) -> Result<T, TypedRequestError>
    where
        T: DeserializeOwned,
    {
        let method = request_method_name(&request);
        Err(TypedRequestError::Transport {
            method,
            source: unsupported_remote_transport(),
        })
    }
}
