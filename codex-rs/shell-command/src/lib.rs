//! Command parsing and safety utilities shared across Codex crates.

pub mod shell_detect;

#[cfg(not(target_arch = "wasm32"))]
pub mod bash;
#[cfg(target_arch = "wasm32")]
#[path = "bash_wasm.rs"]
pub mod bash;
pub(crate) mod command_safety;
pub mod parse_command;
pub mod powershell;

pub use command_safety::is_dangerous_command;
pub use command_safety::is_safe_command;
