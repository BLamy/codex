#[cfg(not(target_arch = "wasm32"))]
pub(crate) use std::time::Instant;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use std::time::SystemTime;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use std::time::UNIX_EPOCH;

#[cfg(target_arch = "wasm32")]
pub(crate) use web_time::Instant;
#[cfg(target_arch = "wasm32")]
pub(crate) use web_time::SystemTime;
#[cfg(target_arch = "wasm32")]
pub(crate) use web_time::UNIX_EPOCH;
