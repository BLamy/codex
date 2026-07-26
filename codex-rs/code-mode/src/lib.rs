#[cfg(not(target_arch = "wasm32"))]
mod cell_actor;
#[cfg(not(target_arch = "wasm32"))]
mod remote_session;
#[cfg(not(target_arch = "wasm32"))]
mod runtime;
#[cfg(not(target_arch = "wasm32"))]
mod service;
#[cfg(not(target_arch = "wasm32"))]
mod session_runtime;
#[cfg(not(target_arch = "wasm32"))]
mod v8_init;
#[cfg(target_arch = "wasm32")]
mod wasm;

pub(crate) type TaskFailureHandler = std::sync::Arc<dyn Fn(String) + Send + Sync>;

pub use codex_code_mode_protocol::*;
#[cfg(not(target_arch = "wasm32"))]
pub use remote_session::ProcessOwnedCodeModeSession;
#[cfg(not(target_arch = "wasm32"))]
pub use remote_session::ProcessOwnedCodeModeSessionProvider;
#[cfg(not(target_arch = "wasm32"))]
pub use service::InProcessCodeModeSession;
#[cfg(not(target_arch = "wasm32"))]
pub use service::InProcessCodeModeSessionProvider;
#[cfg(not(target_arch = "wasm32"))]
pub use service::NoopCodeModeSessionDelegate;
#[cfg(not(target_arch = "wasm32"))]
pub use v8_init::V8JitMode;
#[cfg(not(target_arch = "wasm32"))]
pub use v8_init::initialize_v8;
#[cfg(target_arch = "wasm32")]
pub use wasm::InProcessCodeModeSession;
#[cfg(target_arch = "wasm32")]
pub use wasm::InProcessCodeModeSessionProvider;
#[cfg(target_arch = "wasm32")]
pub use wasm::NoopCodeModeSessionDelegate;
#[cfg(target_arch = "wasm32")]
pub use wasm::ProcessOwnedCodeModeSession;
#[cfg(target_arch = "wasm32")]
pub use wasm::ProcessOwnedCodeModeSessionProvider;
#[cfg(target_arch = "wasm32")]
pub use wasm::V8JitMode;
#[cfg(target_arch = "wasm32")]
pub use wasm::initialize_v8;
