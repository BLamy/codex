#![recursion_limit = "256"]
#![deny(clippy::print_stdout, clippy::print_stderr)]

#[cfg(not(target_arch = "wasm32"))]
include!("lib_native.rs");

#[cfg(target_arch = "wasm32")]
include!("lib_wasm.rs");
