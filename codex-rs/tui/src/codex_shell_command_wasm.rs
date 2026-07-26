fn extract_shell_like_command<'a>(command: &'a [String], shells: &[&str]) -> Option<(&'a str, &'a str)> {
    if command.len() < 3 {
        return None;
    }
    let executable = command.first()?.rsplit('/').next().unwrap_or_default();
    if !shells.iter().any(|shell| executable == *shell) {
        return None;
    }
    let flag_index = command
        .iter()
        .position(|arg| matches!(arg.as_str(), "-c" | "-lc"))?;
    let script = command.get(flag_index + 1)?;
    Some((command.first()?.as_str(), script.as_str()))
}

pub mod bash {
    pub fn extract_bash_command(command: &[String]) -> Option<(&str, &str)> {
        super::extract_shell_like_command(command, &["bash", "sh", "zsh"])
    }
}

pub mod parse_command {
    pub fn extract_shell_command(command: &[String]) -> Option<(&str, &str)> {
        super::extract_shell_like_command(command, &["bash", "sh", "zsh"])
    }

    pub fn shlex_join(tokens: &[String]) -> String {
        tokens
            .iter()
            .map(|token| shlex::try_quote(token).unwrap_or_else(|_| token.as_str().into()))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

pub fn is_dangerous_command(_command: &[String]) -> bool {
    false
}

pub fn is_safe_command(_command: &[String]) -> bool {
    false
}
