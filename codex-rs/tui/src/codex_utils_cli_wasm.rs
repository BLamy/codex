use codex_protocol::ThreadId;
use std::collections::HashMap;

pub fn resume_hint(thread_name: Option<&str>, thread_id: Option<ThreadId>) -> Option<String> {
    match (thread_name, thread_id) {
        (Some(name), Some(id)) if !name.trim().is_empty() => {
            Some(format!("resume with codex resume {id} # {name}"))
        }
        (None, Some(id)) => Some(format!("resume with codex resume {id}")),
        _ => None,
    }
}

pub fn format_env_display(env: &HashMap<String, String>) -> String {
    if env.is_empty() {
        return String::new();
    }
    let mut pairs = env
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>();
    pairs.sort();
    pairs.join(" ")
}
