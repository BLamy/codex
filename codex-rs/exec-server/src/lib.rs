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
mod environment_path;
mod environment_provider;
#[cfg(not(target_arch = "wasm32"))]
mod environment_toml;
#[cfg(not(target_arch = "wasm32"))]
mod fs_helper;
#[cfg(not(target_arch = "wasm32"))]
mod fs_helper_main;
#[cfg(not(target_arch = "wasm32"))]
mod fs_sandbox;
#[cfg(not(target_arch = "wasm32"))]
mod local_file_system;
#[cfg(not(target_arch = "wasm32"))]
mod local_process;
mod process;
mod process_id;
mod protocol;
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
#[cfg(not(target_arch = "wasm32"))]
mod rpc;
mod runtime_paths;
#[cfg(not(target_arch = "wasm32"))]
mod sandboxed_file_system;
#[cfg(not(target_arch = "wasm32"))]
mod server;
#[cfg(target_arch = "wasm32")]
mod wasm_host;

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
pub use client_api::RemoteExecServerConnectArgs;
#[cfg(target_arch = "wasm32")]
pub use client_wasm::ExecServerClient;
#[cfg(target_arch = "wasm32")]
pub use client_wasm::ExecServerError;
#[cfg(target_arch = "wasm32")]
pub use client_wasm::HttpResponseBodyStream;
pub use codex_file_system::CopyOptions;
pub use codex_file_system::CreateDirectoryOptions;
pub use codex_file_system::ExecutorFileSystem;
pub use codex_file_system::FileMetadata;
pub use codex_file_system::FileSystemResult;
pub use codex_file_system::FileSystemSandboxContext;
pub use codex_file_system::ReadDirectoryEntry;
pub use codex_file_system::RemoveOptions;
pub use environment::CODEX_EXEC_SERVER_URL_ENV_VAR;
pub use environment::Environment;
pub use environment::EnvironmentManager;
pub use environment::LOCAL_ENVIRONMENT_ID;
pub use environment::REMOTE_ENVIRONMENT_ID;
pub use environment_path::EnvironmentPathRef;
pub use environment_provider::DefaultEnvironmentProvider;
pub use environment_provider::EnvironmentProvider;
#[cfg(not(target_arch = "wasm32"))]
pub use fs_helper::CODEX_FS_HELPER_ARG1;
#[cfg(not(target_arch = "wasm32"))]
pub use fs_helper_main::main as run_fs_helper_main;
#[cfg(not(target_arch = "wasm32"))]
pub use local_file_system::LOCAL_FS;
#[cfg(not(target_arch = "wasm32"))]
pub use local_file_system::LocalFileSystem;
pub use process::ExecBackend;
pub use process::ExecProcess;
pub use process::ExecProcessEvent;
pub use process::ExecProcessEventReceiver;
pub use process::StartedExecProcess;
pub use process_id::ProcessId;
pub use protocol::ExecClosedNotification;
pub use protocol::ExecEnvPolicy;
pub use protocol::ExecExitedNotification;
pub use protocol::ExecOutputDeltaNotification;
pub use protocol::ExecOutputStream;
pub use protocol::ExecParams;
pub use protocol::ExecResponse;
pub use protocol::FsCanonicalizeParams;
pub use protocol::FsCanonicalizeResponse;
pub use protocol::FsCopyParams;
pub use protocol::FsCopyResponse;
pub use protocol::FsCreateDirectoryParams;
pub use protocol::FsCreateDirectoryResponse;
pub use protocol::FsGetMetadataParams;
pub use protocol::FsGetMetadataResponse;
pub use protocol::FsJoinParams;
pub use protocol::FsJoinResponse;
pub use protocol::FsParentParams;
pub use protocol::FsParentResponse;
pub use protocol::FsReadDirectoryEntry;
pub use protocol::FsReadDirectoryParams;
pub use protocol::FsReadDirectoryResponse;
pub use protocol::FsReadFileParams;
pub use protocol::FsReadFileResponse;
pub use protocol::FsRemoveParams;
pub use protocol::FsRemoveResponse;
pub use protocol::FsWriteFileParams;
pub use protocol::FsWriteFileResponse;
pub use protocol::HttpHeader;
pub use protocol::HttpRequestBodyDeltaNotification;
pub use protocol::HttpRequestParams;
pub use protocol::HttpRequestResponse;
pub use protocol::InitializeParams;
pub use protocol::InitializeResponse;
pub use protocol::ProcessOutputChunk;
pub use protocol::ReadParams;
pub use protocol::ReadResponse;
pub use protocol::TerminateParams;
pub use protocol::TerminateResponse;
pub use protocol::WriteParams;
pub use protocol::WriteResponse;
pub use protocol::WriteStatus;
#[cfg(not(target_arch = "wasm32"))]
pub use remote::RemoteEnvironmentConfig;
#[cfg(not(target_arch = "wasm32"))]
pub use remote::run_remote_environment;
pub use runtime_paths::ExecServerRuntimePaths;
#[cfg(not(target_arch = "wasm32"))]
pub use server::DEFAULT_LISTEN_URL;
#[cfg(not(target_arch = "wasm32"))]
pub use server::ExecServerListenUrlParseError;
#[cfg(not(target_arch = "wasm32"))]
pub use server::run_main;
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
