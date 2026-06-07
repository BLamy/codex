pub use codex_app_server_protocol as protocol;
pub use codex_app_server_transport::AppServerTransport;
pub use codex_app_server_transport::ConnectionId;
pub use codex_app_server_transport::ConnectionOrigin;
pub use codex_app_server_transport::OutgoingError;
pub use codex_app_server_transport::OutgoingMessage;
pub use codex_app_server_transport::OutgoingResponse;
pub use codex_app_server_transport::QueuedOutgoingMessage;
pub use codex_app_server_transport::TransportEvent;
pub use codex_app_server_transport::app_server_control_socket_path;
pub use codex_app_server_transport::app_server_startup_lock_path;

pub mod in_process {
    use std::io::Error as IoError;
    use std::io::ErrorKind;
    use std::io::Result as IoResult;
    use std::sync::Arc;

    use codex_app_server_protocol::ClientNotification;
    use codex_app_server_protocol::ClientRequest;
    use codex_app_server_protocol::ConfigWarningNotification;
    use codex_app_server_protocol::InitializeParams;
    use codex_app_server_protocol::JSONRPCErrorError;
    use codex_app_server_protocol::RequestId;
    use codex_app_server_protocol::Result as JsonRpcResult;
    use codex_app_server_protocol::ServerNotification;
    use codex_app_server_protocol::ServerRequest;
    use codex_arg0::Arg0DispatchPaths;
    use codex_config::CloudConfigBundleLoader;
    use codex_config::LoaderOverrides;
    use codex_config::ThreadConfigLoader;
    use codex_core::config::Config;
    use codex_exec_server::EnvironmentManager;
    use codex_feedback::CodexFeedback;
    use codex_protocol::protocol::SessionSource;
    pub use codex_rollout::StateDbHandle;
    use toml::Value as TomlValue;

    pub const DEFAULT_IN_PROCESS_CHANNEL_CAPACITY: usize =
        codex_app_server_transport::CHANNEL_CAPACITY;

    type PendingClientRequestResponse = std::result::Result<JsonRpcResult, JSONRPCErrorError>;

    #[derive(Clone, Debug)]
    pub struct LogDbLayer;

    #[derive(Clone)]
    pub struct InProcessStartArgs {
        pub arg0_paths: Arg0DispatchPaths,
        pub config: Arc<Config>,
        pub cli_overrides: Vec<(String, TomlValue)>,
        pub loader_overrides: LoaderOverrides,
        pub strict_config: bool,
        pub cloud_config_bundle: CloudConfigBundleLoader,
        pub thread_config_loader: Arc<dyn ThreadConfigLoader>,
        pub feedback: CodexFeedback,
        pub log_db: Option<LogDbLayer>,
        pub state_db: Option<StateDbHandle>,
        pub environment_manager: Arc<EnvironmentManager>,
        pub config_warnings: Vec<ConfigWarningNotification>,
        pub session_source: SessionSource,
        pub enable_codex_api_key_env: bool,
        pub initialize: InitializeParams,
        pub channel_capacity: usize,
    }

    #[derive(Debug)]
    pub enum InProcessServerEvent {
        Lagged { skipped: usize },
        ServerNotification(ServerNotification),
        ServerRequest(ServerRequest),
    }

    #[derive(Clone)]
    pub struct InProcessClientSender;

    pub struct InProcessClientHandle;

    fn unsupported_in_process_runtime() -> IoError {
        IoError::new(
            ErrorKind::Unsupported,
            "in-process app-server runtime is not wired to the wasm host yet",
        )
    }

    impl InProcessClientSender {
        pub async fn request(
            &self,
            _request: ClientRequest,
        ) -> IoResult<PendingClientRequestResponse> {
            Err(unsupported_in_process_runtime())
        }

        pub fn notify(&self, _notification: ClientNotification) -> IoResult<()> {
            Err(unsupported_in_process_runtime())
        }

        pub fn respond_to_server_request(
            &self,
            _request_id: RequestId,
            _result: JsonRpcResult,
        ) -> IoResult<()> {
            Err(unsupported_in_process_runtime())
        }

        pub fn fail_server_request(
            &self,
            _request_id: RequestId,
            _error: JSONRPCErrorError,
        ) -> IoResult<()> {
            Err(unsupported_in_process_runtime())
        }
    }

    impl InProcessClientHandle {
        pub async fn request(
            &self,
            _request: ClientRequest,
        ) -> IoResult<PendingClientRequestResponse> {
            Err(unsupported_in_process_runtime())
        }

        pub fn notify(&self, _notification: ClientNotification) -> IoResult<()> {
            Err(unsupported_in_process_runtime())
        }

        pub fn respond_to_server_request(
            &self,
            _request_id: RequestId,
            _result: JsonRpcResult,
        ) -> IoResult<()> {
            Err(unsupported_in_process_runtime())
        }

        pub fn fail_server_request(
            &self,
            _request_id: RequestId,
            _error: JSONRPCErrorError,
        ) -> IoResult<()> {
            Err(unsupported_in_process_runtime())
        }

        pub async fn next_event(&mut self) -> Option<InProcessServerEvent> {
            None
        }

        pub async fn shutdown(self) -> IoResult<()> {
            Ok(())
        }

        pub fn sender(&self) -> InProcessClientSender {
            InProcessClientSender
        }
    }

    pub async fn start(_args: InProcessStartArgs) -> IoResult<InProcessClientHandle> {
        Err(unsupported_in_process_runtime())
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WasmAppServer;

impl WasmAppServer {
    pub fn new() -> Self {
        Self
    }
}
