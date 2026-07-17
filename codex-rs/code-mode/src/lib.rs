mod cell_actor;
mod remote_session;
#[cfg(not(target_arch = "wasm32"))]
mod runtime;
#[cfg(target_arch = "wasm32")]
mod runtime_wasm;
#[cfg(not(target_arch = "wasm32"))]
mod service;
#[cfg(target_arch = "wasm32")]
mod service_wasm;
mod session_runtime;

pub(crate) type TaskFailureHandler = std::sync::Arc<dyn Fn(String) + Send + Sync>;

pub use codex_code_mode_protocol::*;
pub use remote_session::ProcessOwnedCodeModeSession;
pub use remote_session::ProcessOwnedCodeModeSessionProvider;
#[cfg(not(target_arch = "wasm32"))]
pub use service::InProcessCodeModeSession;
#[cfg(not(target_arch = "wasm32"))]
pub use service::InProcessCodeModeSessionProvider;
#[cfg(not(target_arch = "wasm32"))]
pub use service::NoopCodeModeSessionDelegate;
#[cfg(not(target_arch = "wasm32"))]
pub use service::NotificationFuture;
#[cfg(not(target_arch = "wasm32"))]
pub use service::StartedCell;
#[cfg(not(target_arch = "wasm32"))]
pub use service::ToolInvocationFuture;
#[cfg(target_arch = "wasm32")]
pub use service_wasm::CellId;
#[cfg(target_arch = "wasm32")]
pub use service_wasm::CodeModeService;
#[cfg(target_arch = "wasm32")]
pub use service_wasm::CodeModeSession;
#[cfg(target_arch = "wasm32")]
pub use service_wasm::CodeModeSessionDelegate;
#[cfg(target_arch = "wasm32")]
pub use service_wasm::CodeModeSessionProvider;
#[cfg(target_arch = "wasm32")]
pub use service_wasm::CodeModeSessionProviderFuture;
#[cfg(target_arch = "wasm32")]
pub use service_wasm::CodeModeSessionResultFuture;
#[cfg(target_arch = "wasm32")]
pub use service_wasm::InProcessCodeModeSessionProvider;
#[cfg(target_arch = "wasm32")]
pub use service_wasm::NoopCodeModeSessionDelegate;
#[cfg(target_arch = "wasm32")]
pub use service_wasm::NotificationFuture;
#[cfg(target_arch = "wasm32")]
pub use service_wasm::StartedCell;
#[cfg(target_arch = "wasm32")]
pub use service_wasm::ToolInvocationFuture;
