#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod methods;
mod methods_common;
mod methods_v1;
mod methods_v2;
pub(crate) mod protocol;
mod protocol_common;
mod protocol_v1;
mod protocol_v2;
#[cfg(target_arch = "wasm32")]
pub(crate) mod wasm;

#[cfg(not(target_arch = "wasm32"))]
pub use methods::RealtimeWebsocketClient;
#[cfg(not(target_arch = "wasm32"))]
pub use methods::RealtimeWebsocketConnection;
#[cfg(not(target_arch = "wasm32"))]
pub use methods::RealtimeWebsocketEvents;
#[cfg(not(target_arch = "wasm32"))]
pub use methods::RealtimeWebsocketWriter;
pub use methods_common::session_update_session_json;
pub use protocol::RealtimeEventParser;
pub use protocol::RealtimeOutputModality;
pub use protocol::RealtimeSessionConfig;
pub use protocol::RealtimeSessionMode;
#[cfg(target_arch = "wasm32")]
pub use wasm::RealtimeWebsocketClient;
#[cfg(target_arch = "wasm32")]
pub use wasm::RealtimeWebsocketConnection;
#[cfg(target_arch = "wasm32")]
pub use wasm::RealtimeWebsocketEvents;
#[cfg(target_arch = "wasm32")]
pub use wasm::RealtimeWebsocketWriter;
