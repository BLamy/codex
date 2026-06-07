#[cfg(not(target_arch = "wasm32"))]
pub mod pipe;
#[cfg(not(target_arch = "wasm32"))]
mod process;
#[cfg(not(target_arch = "wasm32"))]
pub mod process_group;
#[cfg(not(target_arch = "wasm32"))]
pub mod pty;
#[cfg(test)]
mod tests;
#[cfg(target_arch = "wasm32")]
mod wasm;
#[cfg(windows)]
mod win;

pub const DEFAULT_OUTPUT_BYTES_CAP: usize = 1024 * 1024;

#[cfg(not(target_arch = "wasm32"))]
/// Spawn a non-interactive process using regular pipes for stdin/stdout/stderr.
pub use pipe::spawn_process as spawn_pipe_process;
#[cfg(not(target_arch = "wasm32"))]
/// Spawn a non-interactive process using regular pipes, but close stdin immediately.
pub use pipe::spawn_process_no_stdin as spawn_pipe_process_no_stdin;
#[cfg(not(target_arch = "wasm32"))]
/// Driver-backed process adapter used by integrations with their own process transport.
pub use process::ProcessDriver;
#[cfg(not(target_arch = "wasm32"))]
/// Handle for interacting with a spawned process (PTY or pipe).
pub use process::ProcessHandle;
#[cfg(not(target_arch = "wasm32"))]
/// Bundle of process handles plus split output and exit receivers returned by spawn helpers.
pub use process::SpawnedProcess;
#[cfg(not(target_arch = "wasm32"))]
/// Terminal size in character cells used for PTY spawn and resize operations.
pub use process::TerminalSize;
#[cfg(not(target_arch = "wasm32"))]
/// Combine stdout/stderr receivers into a single broadcast receiver.
pub use process::combine_output_receivers;
#[cfg(not(target_arch = "wasm32"))]
/// Adapt an externally-driven process into the standard spawned-process handle.
pub use process::spawn_from_driver;
#[cfg(not(target_arch = "wasm32"))]
/// Backwards-compatible alias for ProcessHandle.
pub type ExecCommandSession = ProcessHandle;
#[cfg(not(target_arch = "wasm32"))]
/// Backwards-compatible alias for SpawnedProcess.
pub type SpawnedPty = SpawnedProcess;
#[cfg(not(target_arch = "wasm32"))]
/// Report whether ConPTY is available on this platform (Windows only).
pub use pty::conpty_supported;
#[cfg(not(target_arch = "wasm32"))]
/// Spawn a process attached to a PTY for interactive use.
pub use pty::spawn_process as spawn_pty_process;
#[cfg(target_arch = "wasm32")]
pub use wasm::*;
#[cfg(windows)]
pub use win::PsuedoCon;
#[cfg(windows)]
pub use win::conpty::RawConPty;
