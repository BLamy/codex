//! Conservative shell parsing for the browser runtime.
//!
//! The native implementation uses tree-sitter's C-backed Bash grammar, which
//! is not available to `wasm32-unknown-unknown`. Browser callers can still
//! recognize a shell wrapper for display, but syntax-dependent safety checks
//! deliberately return `None` so an unparsed command is never auto-approved.

/// Opaque placeholder matching the native parser flow within this crate.
pub struct Tree;

pub fn try_parse_shell(_shell_lc_arg: &str) -> Option<Tree> {
    None
}

pub fn try_parse_word_only_commands_sequence(_tree: &Tree, _src: &str) -> Option<Vec<Vec<String>>> {
    None
}

pub fn parse_shell_script_into_commands(_script: &str) -> Option<Vec<Vec<String>>> {
    None
}

pub fn extract_bash_command(command: &[String]) -> Option<(&str, &str)> {
    let [shell, flag, script] = command else {
        return None;
    };
    let executable = shell.rsplit('/').next().unwrap_or_default();
    if !matches!(flag.as_str(), "-lc" | "-c") || !matches!(executable, "bash" | "sh" | "zsh") {
        return None;
    }
    Some((shell, script))
}

pub fn parse_shell_lc_plain_commands(_command: &[String]) -> Option<Vec<Vec<String>>> {
    None
}

pub(crate) fn parse_shell_lc_literal_commands(_command: &[String]) -> Option<Vec<Vec<String>>> {
    None
}

pub fn parse_shell_lc_single_command_prefix(_command: &[String]) -> Option<Vec<String>> {
    None
}
