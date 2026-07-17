#[cfg(not(target_arch = "wasm32"))]
mod client;
mod client_api;
#[cfg(not(target_arch = "wasm32"))]
mod client_transport;
#[cfg(target_arch = "wasm32")]
mod client_wasm;
#[cfg(not(target_arch = "wasm32"))]
mod connection;
mod environment;
mod environment_provider;
mod environment_registry;
#[cfg(not(target_arch = "wasm32"))]
mod environment_toml;
#[cfg(not(target_arch = "wasm32"))]
mod file_read;
mod fs_helper;
#[cfg(not(target_arch = "wasm32"))]
mod fs_helper_main;
#[cfg(not(target_arch = "wasm32"))]
mod fs_sandbox;
#[cfg(not(target_arch = "wasm32"))]
mod local_file_system;
#[cfg(not(target_arch = "wasm32"))]
mod local_process;
mod noise_channel;
mod noise_relay;
mod process;
#[cfg(not(target_arch = "wasm32"))]
mod process_sandbox;
#[cfg(not(target_arch = "wasm32"))]
mod regular_file;
#[cfg(not(target_arch = "wasm32"))]
mod relay;
#[cfg(not(target_arch = "wasm32"))]
mod relay_proto;
#[cfg(not(target_arch = "wasm32"))]
mod remote;
#[cfg(not(target_arch = "wasm32"))]
mod remote_file_system;
#[cfg(not(target_arch = "wasm32"))]
mod remote_process;
mod resolved_capability;
mod rpc;
mod runtime_paths;
#[cfg(not(target_arch = "wasm32"))]
mod sandboxed_file_system;
#[cfg(not(target_arch = "wasm32"))]
mod server;
mod telemetry;
mod trace_context;
#[cfg(target_arch = "wasm32")]
mod wasm_host;
mod websocket_pong_watchdog;

use codex_exec_server_protocol as protocol;

#[cfg(not(target_arch = "wasm32"))]
pub use client::ExecServerClient;
#[cfg(not(target_arch = "wasm32"))]
pub use client::ExecServerError;
#[cfg(not(target_arch = "wasm32"))]
pub use client::http_client::HttpResponseBodyStream;
#[cfg(not(target_arch = "wasm32"))]
pub use client::http_client::ReqwestHttpClient;
pub use client_api::ExecServerClientConnectOptions;
pub use client_api::HttpClient;
pub use client_api::NoiseRendezvousConnectArgs;
pub use client_api::NoiseRendezvousConnectBundle;
pub use client_api::NoiseRendezvousConnectProvider;
pub use client_api::RemoteExecServerConnectArgs;
#[cfg(target_arch = "wasm32")]
pub use client_wasm::ExecServerClient;
#[cfg(target_arch = "wasm32")]
pub use client_wasm::ExecServerError;
#[cfg(target_arch = "wasm32")]
pub use client_wasm::HttpResponseBodyStream;
pub use codex_exec_server_protocol::ProcessId;
pub use codex_file_system::CopyOptions;
pub use codex_file_system::CreateDirectoryOptions;
pub use codex_file_system::ExecutorFileSystem;
pub use codex_file_system::ExecutorFileSystemFuture;
pub use codex_file_system::FILE_READ_CHUNK_SIZE;
pub use codex_file_system::FileMetadata;
pub use codex_file_system::FileSystemReadStream;
pub use codex_file_system::FileSystemResult;
pub use codex_file_system::FileSystemSandboxContext;
pub use codex_file_system::ReadDirectoryEntry;
pub use codex_file_system::RemoveOptions;
pub use codex_file_system::WalkEntry;
pub use codex_file_system::WalkEntryKind;
pub use codex_file_system::WalkError;
pub use codex_file_system::WalkOptions;
pub use codex_file_system::WalkOutcome;
pub use environment::CODEX_EXEC_SERVER_NOISE_AUTH_TOKEN_ENV_VAR;
pub use environment::CODEX_EXEC_SERVER_NOISE_CHATGPT_ACCOUNT_ID_ENV_VAR;
pub use environment::CODEX_EXEC_SERVER_NOISE_ENVIRONMENT_ID_ENV_VAR;
pub use environment::CODEX_EXEC_SERVER_NOISE_REGISTRY_URL_ENV_VAR;
pub use environment::CODEX_EXEC_SERVER_URL_ENV_VAR;
pub use environment::Environment;
pub use environment::EnvironmentManager;
pub use environment::LOCAL_ENVIRONMENT_ID;
pub use environment::REMOTE_ENVIRONMENT_ID;
pub use environment_provider::DefaultEnvironmentProvider;
pub use environment_provider::EnvironmentProvider;
pub use environment_provider::EnvironmentProviderFuture;
pub use environment_registry::EnvironmentRegistryConnectRequest;
pub use environment_registry::EnvironmentRegistryConnectResponse;
pub use environment_registry::EnvironmentRegistryHarnessKeyValidationRequest;
pub use environment_registry::EnvironmentRegistryHarnessKeyValidationResponse;
pub use environment_registry::EnvironmentRegistryRegistrationRequest;
pub use environment_registry::EnvironmentRegistryRegistrationResponse;
pub use fs_helper::CODEX_FS_HELPER_ARG1;
#[cfg(not(target_arch = "wasm32"))]
pub use fs_helper_main::main as run_fs_helper_main;
#[cfg(not(target_arch = "wasm32"))]
pub use local_file_system::LOCAL_FS;
#[cfg(not(target_arch = "wasm32"))]
pub use local_file_system::LocalFileSystem;
pub use noise_channel::NoiseChannelError;
pub use noise_channel::NoiseChannelIdentity;
pub use noise_channel::NoiseChannelPublicKey;
pub use process::ExecBackend;
pub use process::ExecBackendFuture;
pub use process::ExecProcess;
pub use process::ExecProcessEvent;
pub use process::ExecProcessEventReceiver;
pub use process::ExecProcessFuture;
pub use process::StartedExecProcess;
pub use protocol::ByteChunk;
pub use protocol::EnvironmentInfo;
pub use protocol::ExecClosedNotification;
pub use protocol::ExecEnvPolicy;
pub use protocol::ExecExitedNotification;
pub use protocol::ExecOutputDeltaNotification;
pub use protocol::ExecOutputStream;
pub use protocol::ExecParams;
pub use protocol::ExecResponse;
pub use protocol::FsCanonicalizeParams;
pub use protocol::FsCanonicalizeResponse;
pub use protocol::FsCloseParams;
pub use protocol::FsCloseResponse;
pub use protocol::FsCopyParams;
pub use protocol::FsCopyResponse;
pub use protocol::FsCreateDirectoryParams;
pub use protocol::FsCreateDirectoryResponse;
pub use protocol::FsGetMetadataParams;
pub use protocol::FsGetMetadataResponse;
pub use protocol::FsOpenParams;
pub use protocol::FsOpenResponse;
pub use protocol::FsReadBlockParams;
pub use protocol::FsReadBlockResponse;
pub use protocol::FsReadDirectoryEntry;
pub use protocol::FsReadDirectoryParams;
pub use protocol::FsReadDirectoryResponse;
pub use protocol::FsReadFileParams;
pub use protocol::FsReadFileResponse;
pub use protocol::FsRemoveParams;
pub use protocol::FsRemoveResponse;
pub use protocol::FsWalkParams;
pub use protocol::FsWalkResponse;
pub use protocol::FsWriteFileParams;
pub use protocol::FsWriteFileResponse;
pub use protocol::HttpHeader;
pub use protocol::HttpRedirectPolicy;
pub use protocol::HttpRequestBodyDeltaNotification;
pub use protocol::HttpRequestParams;
pub use protocol::HttpRequestResponse;
pub use protocol::InitializeParams;
pub use protocol::InitializeResponse;
pub use protocol::ProcessOutputChunk;
pub use protocol::ProcessSignal;
pub use protocol::ReadParams;
pub use protocol::ReadResponse;
pub use protocol::ShellInfo;
pub use protocol::SignalParams;
pub use protocol::SignalResponse;
pub use protocol::TerminateParams;
pub use protocol::TerminateResponse;
pub use protocol::WriteParams;
pub use protocol::WriteResponse;
pub use protocol::WriteStatus;
#[cfg(not(target_arch = "wasm32"))]
pub use remote::RemoteEnvironmentConfig;
#[cfg(not(target_arch = "wasm32"))]
pub use remote::run_remote_environment;
pub use resolved_capability::ResolvedSelectedCapabilityRoot;
pub use runtime_paths::ExecServerRuntimePaths;
#[cfg(not(target_arch = "wasm32"))]
pub use server::DEFAULT_LISTEN_URL;
#[cfg(not(target_arch = "wasm32"))]
pub use server::ExecServerListenUrlParseError;
#[cfg(not(target_arch = "wasm32"))]
pub use server::run_main;
pub use server::run_main_with_telemetry;
pub use telemetry::ExecServerTelemetry;
#[cfg(target_arch = "wasm32")]
pub use wasm_host::LOCAL_FS;
#[cfg(target_arch = "wasm32")]
pub use wasm_host::WasmHostFileSystem as LocalFileSystem;
#[cfg(target_arch = "wasm32")]
pub use wasm_host::WasmHostHttpClient as ReqwestHttpClient;
#[cfg(target_arch = "wasm32")]
pub use wasm_host::WasmHostProcess as LocalProcess;
#[cfg(target_arch = "wasm32")]
pub use wasm_host::handle_wasm_host_process_event;
