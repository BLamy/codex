pub mod browser_cli;
#[cfg(target_arch = "wasm32")]
pub(crate) mod browser_tui;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod debug_sandbox;
#[cfg(not(target_arch = "wasm32"))]
mod exit_status;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod login;

pub use browser_cli::BrowserCodexCliSession;
pub use browser_cli::BrowserExecPlan;
pub use browser_cli::BrowserLoginMethod;
pub use browser_cli::BrowserLoginRequest;
pub use browser_cli::BrowserRunOptions;
pub use browser_cli::BrowserRunResult;
pub use browser_cli::BrowserTuiAction;
pub use browser_cli::BrowserTuiRunResult;
#[cfg(not(target_arch = "wasm32"))]
use clap::Parser;
#[cfg(not(target_arch = "wasm32"))]
use codex_utils_absolute_path::AbsolutePathBuf;
#[cfg(not(target_arch = "wasm32"))]
use codex_utils_cli::CliConfigOverrides;
#[cfg(not(target_arch = "wasm32"))]
use codex_utils_cli::ProfileV2Name;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

#[cfg(not(target_arch = "wasm32"))]
pub use debug_sandbox::run_command_under_landlock;
#[cfg(not(target_arch = "wasm32"))]
pub use debug_sandbox::run_command_under_seatbelt;
#[cfg(not(target_arch = "wasm32"))]
pub use debug_sandbox::run_command_under_windows_sandbox;
#[cfg(not(target_arch = "wasm32"))]
pub use login::read_access_token_from_stdin;
#[cfg(not(target_arch = "wasm32"))]
pub use login::read_api_key_from_stdin;
#[cfg(not(target_arch = "wasm32"))]
pub use login::run_login_status;
#[cfg(not(target_arch = "wasm32"))]
pub use login::run_login_with_access_token;
#[cfg(not(target_arch = "wasm32"))]
pub use login::run_login_with_api_key;
#[cfg(not(target_arch = "wasm32"))]
pub use login::run_login_with_chatgpt;
#[cfg(not(target_arch = "wasm32"))]
pub use login::run_login_with_device_code;
#[cfg(not(target_arch = "wasm32"))]
pub use login::run_login_with_device_code_fallback_to_browser;
#[cfg(not(target_arch = "wasm32"))]
pub use login::run_logout;

// These command structs share common sandbox options, but remain separate
// because each host backend has a slightly different option surface.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Parser)]
pub struct SeatbeltCommand {
    /// Named permissions profile to apply from the active configuration stack.
    #[arg(long = "permissions-profile", value_name = "NAME")]
    pub permissions_profile: Option<String>,

    /// Layer $CODEX_HOME/<name>.config.toml on top of the base user config.
    #[arg(long = "profile", short = 'p')]
    pub config_profile: Option<ProfileV2Name>,

    /// Working directory used for profile resolution and command execution.
    #[arg(
        short = 'C',
        long = "cd",
        value_name = "DIR",
        requires = "permissions_profile"
    )]
    pub cwd: Option<PathBuf>,

    /// Include managed requirements while resolving an explicit permissions profile.
    #[arg(
        long = "include-managed-config",
        default_value_t = false,
        requires = "permissions_profile"
    )]
    pub include_managed_config: bool,

    /// Allow the sandboxed command to bind/connect AF_UNIX sockets rooted at this path. Relative paths are resolved against the current directory. Repeat to allow multiple paths.
    #[arg(long = "allow-unix-socket", value_parser = parse_allow_unix_socket_path)]
    pub allow_unix_sockets: Vec<AbsolutePathBuf>,

    /// While the command runs, capture macOS sandbox denials via `log stream` and print them after exit
    #[arg(long = "log-denials", default_value_t = false)]
    pub log_denials: bool,

    #[clap(skip)]
    pub config_overrides: CliConfigOverrides,

    /// Full command args to run under seatbelt.
    #[arg(trailing_var_arg = true)]
    pub command: Vec<String>,
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_allow_unix_socket_path(raw: &str) -> Result<AbsolutePathBuf, String> {
    AbsolutePathBuf::relative_to_current_dir(raw)
        .map_err(|err| format!("invalid path {raw}: {err}"))
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Parser)]
pub struct LandlockCommand {
    /// Named permissions profile to apply from the active configuration stack.
    #[arg(long = "permissions-profile", value_name = "NAME")]
    pub permissions_profile: Option<String>,

    /// Layer $CODEX_HOME/<name>.config.toml on top of the base user config.
    #[arg(long = "profile", short = 'p')]
    pub config_profile: Option<ProfileV2Name>,

    /// Working directory used for profile resolution and command execution.
    #[arg(
        short = 'C',
        long = "cd",
        value_name = "DIR",
        requires = "permissions_profile"
    )]
    pub cwd: Option<PathBuf>,

    /// Include managed requirements while resolving an explicit permissions profile.
    #[arg(
        long = "include-managed-config",
        default_value_t = false,
        requires = "permissions_profile"
    )]
    pub include_managed_config: bool,

    #[clap(skip)]
    pub config_overrides: CliConfigOverrides,

    /// Full command args to run under the Linux sandbox.
    #[arg(trailing_var_arg = true)]
    pub command: Vec<String>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Parser)]
pub struct WindowsCommand {
    /// Named permissions profile to apply from the active configuration stack.
    #[arg(long = "permissions-profile", value_name = "NAME")]
    pub permissions_profile: Option<String>,

    /// Layer $CODEX_HOME/<name>.config.toml on top of the base user config.
    #[arg(long = "profile", short = 'p')]
    pub config_profile: Option<ProfileV2Name>,

    /// Working directory used for profile resolution and command execution.
    #[arg(
        short = 'C',
        long = "cd",
        value_name = "DIR",
        requires = "permissions_profile"
    )]
    pub cwd: Option<PathBuf>,

    /// Include managed requirements while resolving an explicit permissions profile.
    #[arg(
        long = "include-managed-config",
        default_value_t = false,
        requires = "permissions_profile"
    )]
    pub include_managed_config: bool,

    #[clap(skip)]
    pub config_overrides: CliConfigOverrides,

    /// Full command args to run under Windows restricted token sandbox.
    #[arg(trailing_var_arg = true)]
    pub command: Vec<String>,
}
